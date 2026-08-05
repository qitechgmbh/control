//! Regenerates `electron/src/machines/dryer/materialPresets.ts` from
//! `machine_implementations/src/dryer/material_presets.rs`, so the material preset data has a
//! single hand-maintained source instead of two copies (Rust + TypeScript) that can silently
//! drift apart.
//!
//! Run with: `cargo run --example gen_material_presets_ts -p machine_implementations`

use machine_implementations::dryer::material_presets::MATERIAL_PRESETS;

fn main() {
    let mut out = String::new();

    out.push_str("// AUTO-GENERATED - DO NOT EDIT BY HAND.\n");
    out.push_str(
        "// Source of truth is machine_implementations/src/dryer/material_presets.rs - edit\n",
    );
    out.push_str("// that file and regenerate with:\n");
    out.push_str("//   cargo run --example gen_material_presets_ts -p machine_implementations\n\n");

    out.push_str("export interface MaterialPreset {\n");
    out.push_str("  abbrev: string;\n");
    out.push_str("  name: string;\n");
    out.push_str("  bulk_density: number;\n");
    out.push_str("  max_moisture_pct: number;\n");
    out.push_str("  temp_min: number;\n");
    out.push_str("  temp_max: number;\n");
    out.push_str("  drying_time_min: number;\n");
    out.push_str("  drying_time_max: number;\n");
    out.push_str("  specific_air_volume: number;\n");
    out.push_str("}\n\n");

    out.push_str("export function recommendedTemp(p: MaterialPreset): number {\n");
    out.push_str("  return Math.floor((p.temp_min + p.temp_max) / 2);\n");
    out.push_str("}\n\n");

    out.push_str("export const MATERIAL_PRESETS: MaterialPreset[] = [\n");
    for p in MATERIAL_PRESETS {
        out.push_str("  {\n");
        out.push_str(&format!("    abbrev: {:?},\n", p.abbrev));
        out.push_str(&format!("    name: {:?},\n", p.name));
        out.push_str(&format!("    bulk_density: {},\n", p.bulk_density));
        out.push_str(&format!("    max_moisture_pct: {},\n", p.max_moisture_pct));
        out.push_str(&format!("    temp_min: {},\n", p.temp_min));
        out.push_str(&format!("    temp_max: {},\n", p.temp_max));
        out.push_str(&format!("    drying_time_min: {},\n", p.drying_time_min));
        out.push_str(&format!("    drying_time_max: {},\n", p.drying_time_max));
        out.push_str(&format!(
            "    specific_air_volume: {},\n",
            p.specific_air_volume
        ));
        out.push_str("  },\n");
    }
    out.push_str("];\n");

    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../electron/src/machines/dryer/materialPresets.ts"
    );
    std::fs::write(path, out).expect("failed to write materialPresets.ts");
    println!("wrote {path}");
}
