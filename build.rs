fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = [
        "proto/google/storage/v2/storage.proto",
        "proto/google/cloud/kms/v1/resources.proto",
        "proto/google/cloud/kms/v1/service.proto",
    ];
    let output = "src/proto";

    tonic_build::configure()
        .out_dir(output)
        .build_server(false)
        .build_client(true)
        .compile(&protos, &["proto"])?;
    Ok(())
}
