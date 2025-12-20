use server::Processor;

mod cache;
mod client;
mod protocol;
mod server;

#[tokio::main]
async fn main() {
    let processor = Processor::build().await.expect("could not startup");
    processor.run_loop().await;
}
