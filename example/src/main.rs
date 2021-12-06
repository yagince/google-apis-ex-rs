use clap::Parser;
use google_apis_ex::{kms::client::KmsClient, storage::client::Client};

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommands,
}

#[derive(Parser)]
enum SubCommands {
    Storage(Storage),
    Kms(Kms),
}

#[derive(Parser)]
struct Storage {
    #[clap(short, long)]
    bucket: String,
    #[clap(short, long)]
    object_id: String,
}

#[derive(Parser)]
struct Kms {
    #[clap(subcommand)]
    subcmd: KmsCommands,
}

#[derive(Parser)]
enum KmsCommands {
    ListKeys(KmsListKeys),
}

#[derive(Parser)]
struct KmsListKeys {
    #[clap(short, long)]
    parent: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommands::Storage(opts) => {
            let client = Client::new().await?;
            dbg!(client.object(&opts.bucket, &opts.object_id).await?);
        }
        SubCommands::Kms(opts) => match opts.subcmd {
            KmsCommands::ListKeys(opts) => {
                let mut client = KmsClient::new().await?;
                dbg!(client.list_key_rings(&opts.parent).await?);
            }
        },
    }

    Ok(())
}
