use std::process::Command;

fn main() {
    let command = Command::new("pnpm")
        .args(&["run", "build"])
        .current_dir("../../frontend");
    // .output()
    // .expect("Failed to build frontend");
}
