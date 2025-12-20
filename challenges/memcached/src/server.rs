use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use log::{error, info};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio::sync::broadcast::Receiver;
use tokio::sync::mpsc;
use tokio::time;

use crate::connection::Connection;
use crate::protocol::{Command, RetrievalCommand};
use crate::store::StoreProcessor;

/// Server listener state. Created in the `run` call. It includes a `run` method
/// which performs the TCP listening and initialization of per-connection state.
struct Listener {
    listener: TcpListener,

    processor: Arc<StoreProcessor>,

    /// Broadcasts a shutdown signal to all active connections.
    ///
    /// The initial `shutdown` trigger is provided by the `run` caller. The
    /// server is responsible for gracefully shutting down active connections.
    /// When a connection task is spawned, it is passed a broadcast receiver
    /// handle. When a graceful shutdown is initiated, a `()` value is sent via
    /// the broadcast::Sender. Each active connection receives it, reaches a
    /// safe terminal state, and completes the task.
    notify_shutdown: broadcast::Sender<()>,

    /// Used as part of the graceful shutdown process to wait for client
    /// connections to complete processing.
    ///
    /// Tokio channels are closed once all `Sender` handles go out of scope.
    /// When a channel is closed, the receiver receives `None`. This is
    /// leveraged to detect all connection handlers completing. When a
    /// connection handler is initialized, it is assigned a clone of
    /// `shutdown_complete_tx`. When the listener shuts down, it drops the
    /// sender held by this `shutdown_complete_tx` field. Once all handler tasks
    /// complete, all clones of the `Sender` are also dropped. This results in
    /// `shutdown_complete_rx.recv()` completing with `None`. At this point, it
    /// is safe to exit the server process.
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl Listener {
    pub async fn run(&mut self) -> std::io::Result<()> {
        info!("accepting inbound connections");
        loop {
            let socket = self.accept().await?;
            let mut handler = Handler {
                con: Connection::new(socket),
                processor: self.processor.clone(),
                shutdown: self.notify_shutdown.subscribe(),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };
            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!("connection error: {:?}", err);
                }
            });
        }
    }

    /// Accept an inbound connection.
    ///
    /// Errors are handled by backing off and retrying. An exponential backoff
    /// strategy is used. After the first failure, the task waits for 1 second.
    /// After the second failure, the task waits for 2 seconds. Each subsequent
    /// failure doubles the wait time. If accepting fails on the 6th try after
    /// waiting for 64 seconds, then this function returns with an error.
    async fn accept(&mut self) -> std::io::Result<TcpStream> {
        let mut backoff = 1;

        // Try to accept a few times
        loop {
            // Perform the accept operation. If a socket is successfully
            // accepted, return it. Otherwise, save the error.
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if backoff > 64 {
                        // Accept has failed too many times. Return the error.
                        return Err(err);
                    }
                }
            }

            // Pause execution until the back off period elapses.
            time::sleep(Duration::from_secs(backoff)).await;

            // Double the back off
            backoff *= 2;
        }
    }
}

struct Handler {
    con: Connection,
    processor: Arc<StoreProcessor>,
    shutdown: Receiver<()>,
    /// Not used directly. Instead, when `Handler` is dropped
    _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    async fn run(&mut self) -> std::io::Result<()> {
        loop {
            tokio::select! {
                com = self.con.read_command() => {
                    let com = com.expect("could not read command");
                    match com {
                        Command::Storage(cmd) => {
                            let no_reply = cmd.no_reply;
                            let res = self.processor.execute_storage_command(cmd).await?;
                            if !no_reply {
                                self.con.write_response(res.to_kw_bytes()).await?;
                            }
                        }
                        Command::Retrieval(cmd) => {
                            match cmd {
                                RetrievalCommand::Get { key } => {
                                    if let Some(val) = self.processor.get(key.as_str()).await {
                                        self.con.write_value(&key, val).await?;
                                    }
                                    self.con.write_response(b"END").await?;
                                }
                            }
                        }
                    }
                }
                _ = self.shutdown.recv() => {
                    return Ok(());
                }
            }
        }
    }
}

pub async fn run(listener: TcpListener, shutdown: impl Future) {
    let processor = Arc::new(StoreProcessor::new());

    // When the provided `shutdown` future completes, we must send a shutdown
    // message to all active connections. We use a broadcast channel for this
    // purpose. The call below ignores the receiver of the broadcast pair, and when
    // a receiver is needed, the subscribe() method on the sender is used to create
    // one.
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, mut shutdown_complete_rx) = mpsc::channel(1);

    let mut server = Listener {
        processor,
        listener,
        notify_shutdown,
        shutdown_complete_tx,
    };

    // Concurrently run the server and listen for the `shutdown` signal. The
    // server task runs until an error is encountered, so under normal
    // circumstances, this `select!` statement runs until the `shutdown` signal
    // is received.
    //
    // `select!` statements are written in the form of:
    //
    // ```
    // <result of async op> = <async op> => <step to perform with result>
    // ```
    //
    // All `<async op>` statements are executed concurrently. Once the **first**
    // op completes, its associated `<step to perform with result>` is
    // performed.
    //
    // The `select!` macro is a foundational building block for writing
    // asynchronous Rust. See the API docs for more details:
    //
    // https://docs.rs/tokio/*/tokio/macro.select.html
    tokio::select! {
        res = server.run() => {
            // // If an error is received here, accepting connections from the TCP
            // // listener failed multiple times and the server is giving up and
            // // shutting down.
            // //
            // // Errors encountered when handling individual connections do not
            // // bubble up to this point.
            if let Err(err) = res {
                panic!("todo: handle error: {:?}", err);
                // error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            // The shutdown signal has been received.
            info!("shutting down");
        }
    }

    // Extract the `shutdown_complete` receiver and transmitter
    // explicitly drop `shutdown_transmitter`. This is important, as the
    // `.await` below would otherwise never complete.
    let Listener {
        notify_shutdown,
        shutdown_complete_tx,
        ..
    } = server;

    // When `notify_shutdown` is dropped, all tasks which have `subscribe`d will
    // receive the shutdown signal and can exit
    drop(notify_shutdown);
    // Drop final `Sender` so the `Receiver` below can complete
    drop(shutdown_complete_tx);

    // Wait for all active connections to finish processing. As the `Sender`
    // handle held by the listener has been dropped above, the only remaining
    // `Sender` instances are held by connection handler tasks. When those drop,
    // the `mpsc` channel will close and `recv()` will return `None`.
    shutdown_complete_rx.recv().await;
}
