mod cli;

#[tokio::main]
async fn main() {
    match cli::main().await {
        Err(err) => println!("{err}"),
        _ => {}
    }
}
