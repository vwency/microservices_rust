fn main() {
    tonic_build::compile_protos("../../proto/auth_service.proto").unwrap();
}
