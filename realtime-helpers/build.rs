use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    eprintln!("Running XDP build.rs now!");

    // Path to your C source
    let src = "src/xdp/xdp_filter_ethercat.c";

    // Output object path in current directory
    let out_obj = PathBuf::from("xdp_eth_filter.o");

    // Run clang to compile eBPF object
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

    let abs_out_obj = env::current_dir()
        .unwrap()
        .join(&out_obj)
        .canonicalize()
        .unwrap();
    println!("cargo:rustc-env=XDP_OBJ_PATH={}", abs_out_obj.display());

    // In build.rs
    use std::fs;
    use std::path::PathBuf;

    let out_file = PathBuf::from(env::var("OUT_DIR").unwrap()).join("xdp_obj_path.txt");
    fs::write(&out_file, abs_out_obj.to_str().unwrap()).unwrap();
}
