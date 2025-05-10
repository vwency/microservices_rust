use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    tonic_build::configure()
        .build_server(true) // Enable server code generation
        .build_client(true) // Enable client code generation
        .out_dir(out_dir)
        .compile_protos(
            &["../../proto/auth_service.proto"],
            &["../../proto"],
        )?;

    Ok(())
}
