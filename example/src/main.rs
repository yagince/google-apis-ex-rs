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
    #[clap(long)]
    output: Option<String>,
}

#[derive(Parser)]
struct Kms {
    #[clap(subcommand)]
    subcmd: KmsCommands,
}

#[derive(Parser)]
enum KmsCommands {
    ListKeyRings(KmsListKeyRings),
}

#[derive(Parser)]
struct KmsListKeyRings {
    #[clap(short, long)]
    parent: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommands::Storage(opts) => {
            let client = Client::new().await?;
            let data = client.object(&opts.bucket, &opts.object_id).await?;

            if let Some(output_path) = opts.output {
                std::fs::write(output_path, data)?;
            } else {
                dbg!(data);
            }
        }
        SubCommands::Kms(opts) => match opts.subcmd {
            KmsCommands::ListKeyRings(opts) => {
                let mut client = KmsClient::new().await?;
                dbg!(client.list_key_rings(&opts.parent).await?);
            }
        },
    }

    Ok(())
}
