use std::{fs, env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=machines");

    let dir = PathBuf::from("../machine-schemas");

    let mut out = String::new();

    out.push_str("pub fn machine_schemas() -> Vec<&'static str> {\n");
    out.push_str("vec![\n");

    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();

        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();

        let escaped = content
            .replace("\\", "\\\\")
            .replace("\"", "\\\"")
            .replace("\n", "\\n");

        out.push_str(&format!(
            "\"{escaped}\",\n"
        ));
    }

    out.push_str("]\n");
    out.push_str("}\n");

    let out_dir = env::var("OUT_DIR").unwrap();
    fs::write(PathBuf::from(out_dir).join("machines.rs"), out).unwrap();
}
