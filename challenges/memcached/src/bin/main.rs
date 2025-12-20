use clap::Parser;
use tokio::net::TcpListener;
use tokio::signal;

#[derive(Parser, Debug)]
#[clap(name = "memcached")]
struct Cli {
    #[clap(short='p', default_value="9999")]
    port: u16
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let args = Cli::parse();
    let listener = TcpListener::bind(&format!("127.0.0.1:{}", args.port)).await?;
    memcached::server::run(listener, signal::ctrl_c()).await;
    Ok(())
}
