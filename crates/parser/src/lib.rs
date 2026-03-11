use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAbi {
    pub contract_id: String,
    pub contract_name: String,
    pub functions: Vec<AbiFunction>,
    pub variables: Vec<AbiVariable>,
    pub maps: Vec<AbiMap>,
    pub fungible_tokens: Vec<String>,
    pub non_fungible_tokens: Vec<AbiNft>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFunction {
    pub name: String,
    pub access: FunctionAccess,
    pub args: Vec<AbiArg>,
    pub outputs: AbiType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FunctionAccess {
    Public,
    ReadOnly,
    Private,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiArg {
    pub name: String,
    pub r#type: AbiType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AbiType {
    Simple(String),
    StringAscii { string_ascii: StringLen },
    StringUtf8 { string_utf8: StringLen },
    Buff { buff: u32 },
    List { list: ListDef },
    Tuple { tuple: Vec<TupleEntry> },
    Optional { optional: Box<AbiType> },
    Response { response: ResponseDef },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLen {
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListDef {
    pub r#type: Box<AbiType>,
    pub length: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TupleEntry {
    pub name: String,
    pub r#type: AbiType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDef {
    pub ok: Box<AbiType>,
    pub error: Box<AbiType>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiVariable {
    pub name: String,
    pub access: String,
    pub r#type: AbiType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiMap {
    pub name: String,
    pub key: AbiType,
    pub value: AbiType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiNft {
    pub name: String,
    pub r#type: AbiType,
}

pub async fn parse_project(contracts_dir: &Path) -> Result<Vec<ContractAbi>> {
    use tokio::process::Command;

    if !contracts_dir.join("Clarinet.toml").exists() {
        return Err(anyhow!(
            "No scaffold-stacks project found. Run from the directory created by stacks-dapp new"
        ));
    }

    // Frontend dir: project root is parent of contracts/, so frontend is sibling of contracts/
    let project_root = contracts_dir
        .parent()
        .ok_or_else(|| anyhow!("Invalid contracts path"))?;
    let frontend_dir = project_root.join("frontend");
    let script = frontend_dir.join("scripts").join("export-abi.mjs");
    if !script.exists() {
        return Err(anyhow!(
            "Frontend ABI script not found at {}. Re-scaffold with stacks-dapp new or add scripts/export-abi.mjs.",
            script.display()
        ));
    }

    let output = Command::new("node")
        .arg(script.as_os_str())
        .current_dir(&frontend_dir)
        .output()
        .await
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                anyhow!("Node.js is required to export ABIs. Install from nodejs.org")
            } else {
                anyhow!("Failed to run export-abi script: {e}")
            }
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "Failed to export contract ABIs. Run clarinet check to validate contracts.\n{}",
            if stderr.is_empty() { "Script exited non-zero." } else { stderr.trim() }
        ));
    }

    let json = String::from_utf8(output.stdout)?;
    parse_abi_list(&json)
}

/// Parse a JSON array of ContractAbi (e.g. from export-abi.mjs stdout).
pub fn parse_abi_list(json: &str) -> Result<Vec<ContractAbi>> {
    let abis: Vec<ContractAbi> = serde_json::from_str(json)?;
    Ok(abis)
}

/// Parse a single ABI JSON string (for testing).
pub fn parse_abi(json: &str) -> Result<ContractAbi> {
    let abi = serde_json::from_str(json)?;
    Ok(abi)
}

/// Map an AbiType into a TypeScript type string.
pub fn abi_type_to_ts(t: &AbiType) -> String {
    match t {
        AbiType::Simple(s) => match s.as_str() {
            "uint128" | "int128" => "bigint".to_string(),
            "bool" => "boolean".to_string(),
            "principal" => "string".to_string(),
            _ => "unknown".to_string(),
        },
        AbiType::StringAscii { .. } | AbiType::StringUtf8 { .. } => "string".to_string(),
        AbiType::Buff { .. } => "Uint8Array".to_string(),
        AbiType::List { list } => {
            let inner = abi_type_to_ts(&list.r#type);
            format!("Array<{inner}>")
        }
        AbiType::Tuple { tuple } => {
            let fields: Vec<String> = tuple
                .iter()
                .map(|e| format!("{}: {}", e.name, abi_type_to_ts(&e.r#type)))
                .collect();
            format!("{{ {} }}", fields.join(", "))
        }
        AbiType::Optional { optional } => format!("{} | null", abi_type_to_ts(optional)),
        AbiType::Response { response } => {
            let ok = abi_type_to_ts(&response.ok);
            let err = abi_type_to_ts(&response.error);
            format!("{{ ok: {ok} }} | {{ error: {err} }}")
        }
    }
}

