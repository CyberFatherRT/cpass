use std::env;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir: PathBuf = env::var("OUT_DIR")?.into();

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("cpass_descriptor.bin"))
        .compile(
            &[
                "proto/auth_service.proto",
                "proto/pass_service.proto",
                "proto/tag_service.proto",
                "proto/types.proto",
            ],
            &["proto"],
        )?;

    Ok(())
}
