mod model;
mod sbom;
mod trivy;

use clap::{Parser, Subcommand};
use std::process::Command as Process;

#[derive(Parser)]
#[command(name = "vex", about = "CycloneDX VEX (Vulnerability Exploitability eXchange) generator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(about = "Create a VEX file from an existing CycloneDX SBOM")]
    Init {
        #[arg(short, long, help = "Path to the CycloneDX SBOM JSON file")]
        sbom: String,
        #[arg(short, long, default_value = "vex.json", help = "Output VEX file path")]
        output: String,
    },
    #[command(
        name = "from-trivy",
        about = "Create a VEX file from a pre-existing Trivy scan and an SBOM"
    )]
    FromTrivy {
        #[arg(short, long, help = "Path to the Trivy JSON output")]
        trivy: String,
        #[arg(short, long, help = "Path to the CycloneDX SBOM JSON file")]
        sbom: String,
        #[arg(short, long, default_value = "vex.json", help = "Output VEX file path")]
        output: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Init { sbom, output } => init(&sbom, &output),
        Command::FromTrivy {
            trivy,
            sbom,
            output,
        } => {
            let bom = sbom::read_sbom(&sbom)?;
            let component = sbom::extract_root_component(&bom)?;
            let trivy_vulns = trivy::read_trivy_report(&trivy)?;
            let vulnerabilities = trivy::match_to_components(&trivy_vulns, &bom.components)?;

            write_vex(&component, &vulnerabilities, &output)
        }
    }
}

fn init(sbom_path: &str, output: &str) -> anyhow::Result<()> {
    let bom = sbom::read_sbom(sbom_path)?;
    let component = sbom::extract_root_component(&bom)?;

    let vulnerabilities = if bom.vulnerabilities.is_empty() {
        let tmp = std::env::temp_dir().join(format!("trivy-{}.json", uuid::Uuid::new_v4()));
        eprintln!("SBOM has no embedded vulnerabilities — running trivy scan...");
        let status = Process::new("trivy")
            .args([
                "sbom",
                "--format",
                "json",
                "--output",
                tmp.to_str().unwrap(),
                sbom_path,
            ])
            .stderr(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .status()?;

        if !status.success() {
            anyhow::bail!("trivy scan failed — is trivy installed?");
        }

        let trivy_vulns = trivy::read_trivy_report(tmp.to_str().unwrap())?;
        let vulns = trivy::match_to_components(&trivy_vulns, &bom.components)?;
        eprintln!("Found {} vulnerabilities via trivy", vulns.len());
        vulns
    } else {
        sbom::stub_vulnerabilities(&bom)
    };

    write_vex(&component, &vulnerabilities, output)
}

fn write_vex(
    component: &model::Component,
    vulnerabilities: &[model::Vulnerability],
    output: &str,
) -> anyhow::Result<()> {
    let vex = model::Bom::new_vex(component.clone(), vulnerabilities.to_vec());
    let json = serde_json::to_string_pretty(&vex)?;
    std::fs::write(output, json)?;
    println!("VEX written to {output}");
    Ok(())
}
