use std::env;
use std::path::PathBuf;

fn main() {
    tonic_build::configure()
        .out_dir(PathBuf::from(env::var("OUT_DIR").unwrap()))
        .compile(&["../../proto/auth_service.proto"], &["../../proto"])
        .expect("Failed to compile protos");
}
