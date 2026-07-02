use std::{env, fs, path::PathBuf};

use machine_core::MachineSchema;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let file_path = PathBuf::from(out_dir).join("generated.rs");

    let mut out = String::new();

    out.push_str("mod generated {");

    for entry in fs::read_dir("../machine-schemas").unwrap() {
        let path = entry.unwrap().path();

        if path.extension().and_then(|e| e.to_str()) != Some("yaml") {
            continue;
        }

        let content = fs::read_to_string(&path).unwrap();

        process_machine_schema(content, &mut out);
    }

    out.push_str("}");

    panic!("oh no bro!");
}

fn process_machine_schema(content: String, out: &mut String) {
    let schema = yaml_serde::from_str::<MachineSchema>(&content).unwrap();

    println!("schema: {schema:?}");
}

// generated::machines::laser_v1::