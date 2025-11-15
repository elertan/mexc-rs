use std::ops::Deref;
use std::sync::LazyLock;
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const WS_PROTO_SUB_NAME: &str = "websocket-proto";
static MANIFEST_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
});

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let ws_proto_dir = MANIFEST_DIR.join(WS_PROTO_SUB_NAME);

    if !ws_proto_dir.exists() {
        run_command_checked("git", &["submodule", "init", WS_PROTO_SUB_NAME]);
        run_command_checked("git", &["submodule", "update", WS_PROTO_SUB_NAME]);
    }

    if !ws_proto_dir.exists() {
        panic!(
            "Directory {} does not exist after initializing submodule",
            ws_proto_dir.display()
        );
    }

    let proto_files =
        collect_proto_files(&ws_proto_dir).expect("failed to read proto files in websocket-proto");

    if proto_files.is_empty() {
        panic!("No .proto files found in {}", ws_proto_dir.display());
    }

    // rerun the build script if any of the proto files change
    for proto in &proto_files {
        if let Ok(rel) = proto.strip_prefix(MANIFEST_DIR.deref()) {
            println!("cargo:rerun-if-changed={}", rel.display());
        } else {
            println!("cargo:rerun-if-changed={}", proto.display());
        }
        println!("{}", proto.display())
    }

    tonic_build::configure()
        .compile_protos(&proto_files, &[ws_proto_dir])
        .unwrap();
}

fn collect_proto_files(dir: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            let next = collect_proto_files(&path)?;
            out.extend(next);
        } else if path.extension().map_or(false, |ext| ext == "proto") {
            out.push(path);
        }
    }
    Ok(out)
}

fn run_command_checked(cmd: &str, args: &[&str]) {
    let status = std::process::Command::new(cmd)
        .args(args)
        .current_dir(MANIFEST_DIR.deref())
        .status()
        .expect("failed to execute git to initialize websocket-proto submodule");

    let mut args_str = String::new();
    args.iter().for_each(|arg| {
        args_str.push_str(arg);
        args_str.push(' ');
    });
    if !status.success() {
        panic!("Command failed: {cmd} {args_str}");
    }
}
