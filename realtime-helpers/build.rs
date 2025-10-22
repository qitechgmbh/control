use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    eprintln!("Running XDP build.rs now!");

    let src = "src/xdp/xdp_filter_ethercat.c";
    let out_obj = PathBuf::from("xdp_eth_filter.o");

    // Compile the XDP object
    let status = Command::new("clang")
        .args([
            "-O2",
            "-Wall",
            "-target",
            "bpf",
            "-I/usr/include",
            "-I/usr/include/x86_64-linux-gnu",
            "-D__TARGET_ARCH_x86",
            "-c",
            src,
            "-o",
            out_obj.to_str().unwrap(),
        ])
        .status()
        .expect("failed to run clang");

    if !status.success() {
        panic!("clang failed with exit code {}", status);
    }

    println!("cargo:rerun-if-changed={}", src);

    // Compute absolute path
    let abs_out_obj = env::current_dir().unwrap().join(&out_obj).canonicalize().unwrap();

    // Export path as environment variable (runtime access)
    println!("cargo:rustc-env=XDP_OBJ_PATH={}", abs_out_obj.display());

    // Also generate a Rust file with a constant (compile-time access)
    let out_file = PathBuf::from(env::var("OUT_DIR").unwrap()).join("xdp_obj_path.rs");
    fs::write(
        &out_file,
        format!("pub const XDP_OBJ_PATH: &str = {:?};", abs_out_obj),
    ).unwrap();
}
