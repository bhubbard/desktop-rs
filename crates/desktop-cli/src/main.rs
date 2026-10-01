#[tokio::main]
async fn main() -> anyhow::Result<()> {
    desktop_cli::run().await
}
