use clap::Parser;
use google_apis_ex::storage::client::Client;

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommands,
}

#[derive(Parser)]
enum SubCommands {
    Storage(Storage),
}

#[derive(Parser)]
struct Storage {
    #[clap(short, long)]
    bucket: String,
    #[clap(short, long)]
    object_id: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommands::Storage(opts) => {
            let client = Client::new().await?;
            dbg!(client.object(&opts.bucket, &opts.object_id).await?);
        }
    }

    Ok(())
}
