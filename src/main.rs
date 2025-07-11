mod cli;
use packtrack::error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::main().await
}
