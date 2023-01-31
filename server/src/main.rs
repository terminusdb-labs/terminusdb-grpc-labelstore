use clap::Parser;
mod server;

use server::*;
use terminus_store::storage::directory::CachedDirectoryLabelStore;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    store_location: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let store = CachedDirectoryLabelStore::open(cli.store_location).await?;

    spawn_server(store).await?;

    Ok(())
}
