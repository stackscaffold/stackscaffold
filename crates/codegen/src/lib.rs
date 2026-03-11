use anyhow::Result;
use parser::ContractAbi;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tera::Tera;

const CONTRACTS_TS_TEMPLATE: &str =
    include_str!("../../../templates/contracts.ts.tera");
const HOOKS_TS_TEMPLATE: &str = include_str!("../../../templates/hooks.ts.tera");
const DEBUG_UI_TSX_TEMPLATE: &str =
    include_str!("../../../templates/debug_ui.tsx.tera");

/// Main entry called by CLI `generate` command.
pub async fn generate_all() -> Result<()> {
    let project_root = std::env::current_dir()?;
    let contracts_dir = project_root.join("contracts");
    if !contracts_dir.join("Clarinet.toml").exists() || !project_root.join("frontend/package.json").exists() {
        anyhow::bail!(
            "No scaffold-stacks project found. Run from the directory created by stacks-dapp new"
        );
    }

    // Ensure frontend deps so export-abi.mjs can run
    let frontend_dir = project_root.join("frontend");
    if !frontend_dir.join("node_modules").exists() {
        println!("Installing frontend dependencies (npm install)...");
        let status = tokio::process::Command::new("npm")
            .arg("install")
            .current_dir(&frontend_dir)
            .status()
            .await?;
        if !status.success() {
            anyhow::bail!("npm install in frontend failed. Fix errors above and re-run stacks-dapp generate.");
        }
    }

    let abis = parser::parse_project(&contracts_dir).await?;

    let out_dir = project_root.join("frontend/src/generated");
    tokio::fs::create_dir_all(&out_dir).await?;

    render(&abis, &out_dir)?;
    Ok(())
}

#[derive(Serialize)]
struct TemplateContract<'a> {
    #[serde(flatten)]
    inner: &'a ContractAbi,
}

/// Render all templates into `out_dir`.
pub fn render(abis: &[ContractAbi], out_dir: &Path) -> Result<()> {
    let mut tera = Tera::default();
    tera.add_raw_template("contracts.ts.tera", CONTRACTS_TS_TEMPLATE)?;
    tera.add_raw_template("hooks.ts.tera", HOOKS_TS_TEMPLATE)?;
    tera.add_raw_template("debug_ui.tsx.tera", DEBUG_UI_TSX_TEMPLATE)?;

    let contracts: Vec<TemplateContract> = abis.iter().map(|c| TemplateContract { inner: c }).collect();

    let ctx = tera::Context::from_serialize(serde_json::json!({ "contracts": contracts }))?;

    write_if_changed(out_dir.join("contracts.ts"), &tera.render("contracts.ts.tera", &ctx)?)?;
    write_if_changed(out_dir.join("hooks.ts"), &tera.render("hooks.ts.tera", &ctx)?)?;
    write_if_changed(
        out_dir.join("DebugContracts.tsx"),
        &tera.render("debug_ui.tsx.tera", &ctx)?,
    )?;

    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher.finalize().to_vec()
}

fn write_if_changed(path: PathBuf, contents: &str) -> Result<()> {
    let new_bytes = contents.as_bytes();
    let new_hash = hash_bytes(new_bytes);

    if let Ok(existing) = fs::read(&path) {
        if hash_bytes(&existing) == new_hash {
            return Ok(());
        }
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(&path)?;
    file.write_all(new_bytes)?;
    println!("[generated] {}", path.display());
    Ok(())
}

