use clap::Parser;
mod server;

use server::*;
use terminus_store::storage::directory::CachedDirectoryLabelStore;

#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    /// The location of the label store
    store_location: String,
    /// The port to run on
    #[arg(short='p',long="port",default_value_t=8080)]
    port: u16
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let store = CachedDirectoryLabelStore::open(cli.store_location).await?;

    spawn_server(store, cli.port).await?;

    Ok(())
}
