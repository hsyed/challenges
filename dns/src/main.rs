use server::Processor;

mod protocol;
mod server;
mod cache;

#[tokio::main]
async fn main() {
    let processor = Processor::build().
        await.expect("could not startup");
    processor.run_loop().await;
}
