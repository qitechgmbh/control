use std::{collections::HashMap, env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=machines");

    let out_dir = env::var("OUT_DIR").unwrap();
    let file_path = PathBuf::from(out_dir).join("generated.rs");
    let mut out = String::new();

    out.push_str("mod generated {\n");
    out.push_str("use std::collections::HashMap;\n\n");
    inject_schemas("../machine-schemas", &mut out);
    inject_vendors("../vendors.json", &mut out);
    out.push('}');

    fs::write(&file_path, out).unwrap();
}

fn inject_schemas(dir: &str, out: &mut String) {
    out.push_str("pub fn machine_schemas() -> Vec<&'static str> {\n");
    out.push_str("vec![\n");

    for entry in fs::read_dir(dir).unwrap() {
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
}

fn inject_vendors(path: &str, out: &mut String) {
    out.push_str("pub fn vendors() -> HashMap<u16, &'static str> {\n");
    out.push_str("    HashMap::from([\n");

    let data = fs::read_to_string(path).unwrap();
    let vendors: HashMap<String, String> =
        serde_json::from_str(&data).unwrap();

    for (id, name) in vendors {
        let id = if let Some(hex) = id.strip_prefix("0x") {
            u16::from_str_radix(hex, 16).unwrap()
        } else {
            u16::from_str_radix(&id, 16).unwrap_or_else(|_| id.parse().unwrap())
        };

        out.push_str(&format!("        ({}, {:?}),\n", id, name));
    }

    out.push_str("    ])\n");
    out.push_str("}\n");
}
