use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir: PathBuf = env::var("OUT_DIR")?.into();

    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .file_descriptor_set_path(out_dir.join("cpass_descriptor.bin"))
        .compile(
            &[
                "proto/types.proto",
                "proto/auth_service.proto",
                "proto/pass_service.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
