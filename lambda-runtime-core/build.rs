use rustc_version::{Channel, VersionMeta};
use std::{env, fs, io::Write, path::Path, process::Command};

const RUNTIME_METADATA_FILE: &str = "runtime_release";

fn main() {
    println!("Generating AWS Lambda metadata file");
    let out_dir = env::var("OUT_DIR").unwrap();
    let compiler = env::var("RUSTC").unwrap();
    let cargo_version = env::var("CARGO_PKG_VERSION").unwrap();
    let compiler_version =
        VersionMeta::for_command(Command::new(compiler.clone())).expect("Could not load compiler metdata");
    let chn: &str;
    match compiler_version.channel {
        Channel::Dev => chn = "dev",
        Channel::Nightly => chn = "nightly",
        Channel::Beta => chn = "beta",
        Channel::Stable => chn = "stable",
    }
    let compiler_str = format!("{}/{}-{}", compiler, compiler_version.semver, chn);

    let agent = format!("AWS_Lambda_Rust/{} ({})", cargo_version, compiler_str);
    // we expect this library to be built as a dependency and the output directory
    // to be something like: my-lambda-function/target/release/build/lambda_runtime_core-c1abe336a4420096/out.
    // we want the metadata file to be generated alongside the executable of the function
    // so we travel 3 directories up to my-lambda-function/target/release.
    let metadata_path = Path::new(&out_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(RUNTIME_METADATA_FILE);
    println!("Writing runtime metadata to: {}", metadata_path.to_str().unwrap());
    println!("Runtime metadata: {}", agent);
    fs::write(metadata_path, agent.clone()).expect("Could not write runtime metdata file");

    // next generate the metadata function for the runtime
    let dest_path = Path::new(&out_dir).join("metadata.rs");
    let mut f = fs::File::create(&dest_path).unwrap();

    f.write_all(
        format!(
            "
/// returns metdata information about the Lambda runtime
pub fn runtime_release() -> &'static str {{
    \"{}\"
}}
",
            agent
        )
        .as_bytes(),
    )
    .unwrap();
}
