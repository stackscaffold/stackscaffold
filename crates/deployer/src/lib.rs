use anyhow::{anyhow, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

pub struct NetworkConfig {
    pub stacks_node: String,
    pub bitcoin_node: String,
}

pub fn network_config(network: &str) -> NetworkConfig {
    match network {
        "devnet" => NetworkConfig {
            stacks_node: "http://localhost:3999".into(),
            bitcoin_node: "http://localhost:18443".into(),
        },
        "testnet" => NetworkConfig {
            stacks_node: "https://api.testnet.hiro.so".into(),
            bitcoin_node: "https://blockstream.info/testnet/api".into(),
        },
        "mainnet" => NetworkConfig {
            stacks_node: "https://api.hiro.so".into(),
            bitcoin_node: "https://blockstream.info/api".into(),
        },
        other => panic!("Unknown network: {other}"),
    }
}

#[derive(Serialize)]
struct DeploymentInfo {
    contract_id: String,
    tx_id: String,
    block_height: u64,
}

#[derive(Serialize)]
struct DeploymentFile {
    network: String,
    deployed_at: String,
    contracts: HashMap<String, DeploymentInfo>,
}

pub async fn deploy(network: &str) -> Result<()> {
    let config = network_config(network);
    println!("Deploying contracts to {}", config.stacks_node);

    // Minimal placeholder: scan contracts/contracts for .clar files and
    // write a fake deployments.json with predictable IDs.
    let contracts_root = Path::new("contracts/contracts");
    if !contracts_root.exists() {
        return Err(anyhow!(
            "No scaffold-stacks project found. Run from the directory created by stacks-dapp new"
        ));
    }

    let mut contracts_map = HashMap::new();
    let mut entries = fs::read_dir(contracts_root).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("clar") {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                let contract_id = format!("FAKE.{stem}");
                let tx_id = "0xdeadbeef".to_string();
                let info = DeploymentInfo {
                    contract_id: contract_id.clone(),
                    tx_id: tx_id.clone(),
                    block_height: 0,
                };
                contracts_map.insert(stem.to_string(), info);
                println!("contract {stem} | txid {tx_id} | address {contract_id}");
            }
        }
    }

    let deployments = DeploymentFile {
        network: network.to_string(),
        deployed_at: chrono::Utc::now().to_rfc3339(),
        contracts: contracts_map,
    };

    let json = serde_json::to_string_pretty(&deployments)?;
    let out_path = Path::new("frontend/src/generated/deployments.json");
    if let Some(parent) = out_path.parent() {
        fs::create_dir_all(parent).await?;
    }
    fs::write(out_path, json).await?;

    Ok(())
}

