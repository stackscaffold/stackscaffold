use anyhow::{anyhow, Result};
use include_dir::{include_dir, Dir};
use std::path::Path;
use tokio::process::Command;
use which::which;

static FRONTEND_TEMPLATE: Dir =
    include_dir!("$CARGO_MANIFEST_DIR/../../frontend-template");

pub async fn new_project(name: &str, git_init: bool) -> Result<()> {
    println!("⚡ scaffold-stacks — creating {name}");

    ensure_prerequisites().await?;

    let root = Path::new(name);
    if root.exists() {
        return Err(anyhow!("Target directory '{}' already exists", name));
    }
    tokio::fs::create_dir_all(root).await?;

    // Copy frontend template into <name>/frontend
    let frontend_dir = root.join("frontend");
    // Ensure the target directory exists before extracting files.
    tokio::fs::create_dir_all(&frontend_dir).await?;
    FRONTEND_TEMPLATE
        .extract(&frontend_dir)
        .map_err(|e| anyhow!("Failed to copy frontend template: {e}"))?;

    // Contracts layout
    let contracts_root = root.join("contracts");
    tokio::fs::create_dir_all(contracts_root.join("contracts")).await?;
    tokio::fs::create_dir_all(contracts_root.join("settings")).await?;
    tokio::fs::create_dir_all(contracts_root.join("tests")).await?;

    // contracts/package.json — run contract tests with Vitest (Clarinet v2+)
    let contracts_package = r#"{
  "name": "contracts",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "vitest run"
  },
  "devDependencies": {
    "@hirosystems/clarinet-sdk": "^2",
    "@stacks/transactions": "^6",
    "typescript": "^5",
    "vitest": "^1"
  }
}
"#;
    tokio::fs::write(contracts_root.join("package.json"), contracts_package).await?;

    // contracts/vitest.config.ts
    let vitest_config = r#"import { defineConfig } from 'vitest/config';
export default defineConfig({
  test: { environment: 'node' },
});
"#;
    tokio::fs::write(contracts_root.join("vitest.config.ts"), vitest_config).await?;

    // contracts/tsconfig.json
    let contracts_tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "skipLibCheck": true
  },
  "include": ["tests/**/*.ts"]
}
"#;
    tokio::fs::write(contracts_root.join("tsconfig.json"), contracts_tsconfig).await?;

    // Clarinet.toml
    let clarinet_toml = format!(
        "[project]
name = \"{name}\"
description = \"\"
authors = []
telemetry = false
cache_dir = \"./.cache\"

[contracts.counter]
path = \"contracts/counter.clar\"
clarity_version = 3
epoch = \"3.0\"

[repl.costs_version]
version = 2
"
    );
    tokio::fs::write(contracts_root.join("Clarinet.toml"), clarinet_toml).await?;

    // Network settings placeholders
    tokio::fs::write(contracts_root.join("settings/Devnet.toml"), "").await?;
    tokio::fs::write(contracts_root.join("settings/Testnet.toml"), "").await?;
    tokio::fs::write(contracts_root.join("settings/Mainnet.toml"), "").await?;

    // counter.clar
    let counter_clar = r#";; counter.clar — sample contract scaffolded by scaffold-stacks

(define-data-var counter uint u0)

(define-read-only (get-count)
  (ok (var-get counter)))

(define-public (increment)
  (begin
    (var-set counter (+ (var-get counter) u1))
    (ok (var-get counter))))

(define-public (decrement)
  (begin
    (asserts! (> (var-get counter) u0) (err u1))
    (var-set counter (- (var-get counter) u1))
    (ok (var-get counter))))

(define-public (reset)
  (begin
    (var-set counter u0)
    (ok u0)))
"#;
    tokio::fs::write(contracts_root.join("contracts/counter.clar"), counter_clar).await?;

    // counter.test.ts — run with npm test from contracts/
    let counter_test = r#"import { describe, expect, it } from 'vitest';
import { initSimnet } from '@hirosystems/clarinet-sdk';
import { Cl } from '@stacks/transactions';

const simnet = await initSimnet();
const accounts = simnet.getAccounts();
const address1 = accounts.get('wallet_1')!;

describe('counter', () => {
  it('increments', () => {
    const { result } = simnet.callPublicFn('counter', 'increment', [], address1);
    expect(result).toBeOk(Cl.uint(1));
  });
  it('get-count returns current value', () => {
    const { result } = simnet.callReadOnlyFn('counter', 'get-count', [], address1);
    expect(result).toBeOk(Cl.uint(1));
  });
  it('decrement', () => {
    const { result } = simnet.callPublicFn('counter', 'decrement', [], address1);
    expect(result).toBeOk(Cl.uint(0));
  });
});
"#;
    tokio::fs::write(contracts_root.join("tests/counter.test.ts"), counter_test).await?;

    // root package.json
    let root_package = format!(
        "{{\n  \"name\": \"{name}\",\n  \"private\": true,\n  \"scripts\": {{\n    \"dev\": \"stacks-dapp dev\",\n    \"generate\": \"stacks-dapp generate\",\n    \"deploy\": \"stacks-dapp deploy\",\n    \"test\": \"stacks-dapp test\",\n    \"check\": \"stacks-dapp check\"\n  }}\n}}\n"
    );
    tokio::fs::write(root.join("package.json"), root_package).await?;

    // npm install in frontend
    Command::new("npm")
        .arg("install")
        .current_dir(&frontend_dir)
        .spawn()?
        .wait()
        .await?;

    // npm install in contracts (for vitest + clarinet-sdk tests)
    Command::new("npm")
        .arg("install")
        .current_dir(&contracts_root)
        .spawn()?
        .wait()
        .await?;

    if git_init {
        Command::new("git")
            .arg("init")
            .current_dir(root)
            .spawn()?
            .wait()
            .await?;
        Command::new("git")
            .args(["add", "-A"])
            .current_dir(root)
            .spawn()?
            .wait()
            .await?;
        Command::new("git")
            .args(["commit", "-m", "scaffold-stacks init"])
            .current_dir(root)
            .spawn()?
            .wait()
            .await?;
    }

    println!("✅ Project '{name}' created. Next steps:\n  cd {name}\n  stacks-dapp dev");
    Ok(())
}

pub async fn add_contract(name: &str, _template: &str) -> Result<()> {
    let path = Path::new("contracts/contracts").join(format!("{name}.clar"));
    if path.exists() {
        return Err(anyhow!("Contract '{}' already exists", name));
    }
    let contents = format!(
        ";; {name}.clar\n\n(define-read-only (get-info)\n  (ok \"{name} contract\"))\n"
    );
    tokio::fs::write(&path, contents).await?;
    println!("[added] {} — bindings regeneration not yet implemented", path.display());
    Ok(())
}

async fn ensure_prerequisites() -> Result<()> {
    if which("node").is_err() {
        return Err(anyhow!(
            "Node.js >=20 is required. Install from nodejs.org"
        ));
    }
    if which("clarinet").is_err() {
        return Err(anyhow!(
            "clarinet is required. Install: brew install clarinet  OR  cargo install clarinet"
        ));
    }
    Ok(())
}

