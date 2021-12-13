use clap::Parser;
use google_apis_ex::{kms::client::KmsClient, pubsub::PubSubClient, storage::client::Client};

#[derive(Parser)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommands,
}

#[derive(Parser)]
enum SubCommands {
    Storage(Storage),
    Kms(Kms),
    Pubsub(Pubsub),
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
    ListKeyRings(KmsListKeysParams),
    ListCryptoKeys(KmsListKeysParams),
    Encrypt(KmsEncryptParams),
}

#[derive(Parser)]
struct KmsListKeysParams {
    #[clap(short, long)]
    parent: String,
}

#[derive(Parser)]
struct KmsEncryptParams {
    #[clap(short, long)]
    key: String,
    #[clap(short, long)]
    data: String,
}

#[derive(Parser)]
struct Pubsub {
    #[clap(subcommand)]
    subcmd: PubsubSubCommands,
}

#[derive(Parser)]
enum PubsubSubCommands {
    Publish(PubsubPublishParams),
    Pull(PubsubPullParams),
}

#[derive(Parser)]
struct PubsubPublishParams {
    #[clap(short, long)]
    topic: String,
    #[clap(short, long)]
    data: String,
}

#[derive(Parser)]
struct PubsubPullParams {
    #[clap(short, long)]
    subscription: String,
    #[clap(short, long)]
    ack: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = Opts::parse();

    match opts.subcmd {
        SubCommands::Storage(opts) => {
            let mut client = Client::new().await?;
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
            KmsCommands::ListCryptoKeys(opts) => {
                let mut client = KmsClient::new().await?;
                dbg!(client.list_crypto_keys(&opts.parent).await?);
            }
            KmsCommands::Encrypt(opts) => {
                let mut client = KmsClient::new().await?;
                let res = client.encrypt(&opts.key, opts.data.as_bytes()).await?;

                println!("encrypt success.");

                let res = client.decrypt(&opts.key, res.ciphertext).await?;

                println!("decrypt success.");
                println!("decrepted: {}", String::from_utf8(res.plaintext)?);
            }
        },
        SubCommands::Pubsub(opts) => match opts.subcmd {
            PubsubSubCommands::Publish(opts) => {
                let mut client = PubSubClient::new().await?;
                let res = client.publish(&opts.topic, opts.data.as_bytes()).await?;
                dbg!(res);
            }
            PubsubSubCommands::Pull(opts) => {
                let mut client = PubSubClient::new().await?;
                let res = dbg!(client.pull(&opts.subscription).await?);
                for message in res.received_messages.into_iter() {
                    if let Some(mes) = message.message {
                        dbg!(mes.message_id);
                        let _ = dbg!(String::from_utf8(mes.data));

                        if opts.ack {
                            println!("Ack {:?}", message.ack_id);
                            client
                                .acknowledge(&opts.subscription, vec![message.ack_id])
                                .await?;
                        }
                    }
                }
            }
        },
    }

    Ok(())
}
