use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(name = "stacks-dapp", version, about = "Scaffold-Stacks CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a new monorepo workspace
    New {
        /// Project name (becomes directory name)
        name: String,
        /// Skip git init
        #[arg(long)]
        no_git: bool,
    },
    /// Start devnet + frontend + file watcher
    Dev,
    /// Parse contracts and regenerate TypeScript bindings
    Generate,
    /// Add a new Clarity contract to the workspace
    Add {
        #[command(subcommand)]
        entity: AddEntity,
    },
    /// Deploy contracts to a network
    Deploy {
        #[arg(long, default_value = "testnet")]
        network: String,
    },
    /// Run contract tests (Vitest in contracts/) and frontend tests (vitest)
    Test,
    /// Type-check all Clarity contracts
    Check,
    /// Remove generated files and devnet state
    Clean,
}

#[derive(Subcommand)]
enum AddEntity {
    /// Add a new smart contract
    Contract {
        name: String,
        #[arg(long, default_value = "blank")]
        template: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::New { name, no_git } => scaffold::new_project(&name, !no_git).await,
        Commands::Dev => process_supervisor::dev().await,
        Commands::Generate => codegen::generate_all().await,
        Commands::Add { entity } => handle_add(entity).await,
        Commands::Deploy { network } => deployer::deploy(&network).await,
        Commands::Test => run_test().await,
        Commands::Check => run_check().await,
        Commands::Clean => run_clean().await,
    }
}

async fn handle_add(entity: AddEntity) -> Result<()> {
    match entity {
        AddEntity::Contract { name, template } => scaffold::add_contract(&name, &template).await,
    }
}

async fn run_test() -> Result<()> {
    use tokio::process::Command;

    // Contract tests: npm run test in contracts/ (Vitest + Clarinet SDK; no "clarinet test")
    println!("{}", "[test] Running contract tests (Vitest)...".cyan());
    if tokio::fs::metadata("contracts/node_modules").await.is_err() {
        println!("{}", "[test] Installing contract dependencies (npm install in contracts/)...".cyan());
        let install_status = Command::new("npm")
            .arg("install")
            .current_dir("contracts")
            .status()
            .await;
        match install_status {
            Ok(s) if s.success() => println!("{}", "[test] contracts npm install done.".green()),
            Ok(_) => anyhow::bail!("npm install failed in contracts/. Fix errors and retry."),
            Err(_) => anyhow::bail!("Node.js is required. Install from nodejs.org"),
        }
    }
    let contract_status = Command::new("npm")
        .args(["run", "test"])
        .current_dir("contracts")
        .status()
        .await;
    match contract_status {
        Ok(s) if !s.success() => anyhow::bail!("Contract tests failed. Fix the errors above."),
        Err(_) => anyhow::bail!("Failed to run contract tests. Ensure Node.js is installed."),
        Ok(_) => println!("{}", "[test] Contract tests passed.".green()),
    }

    // Frontend tests: npm run test in frontend/
    if tokio::fs::metadata("frontend/node_modules").await.is_err() {
        println!("{}", "[test] Installing frontend dependencies (npm install)...".cyan());
        let install_status = Command::new("npm")
            .arg("install")
            .current_dir("frontend")
            .status()
            .await;
        match install_status {
            Ok(s) if s.success() => println!("{}", "[test] npm install completed.".green()),
            Ok(_) => anyhow::bail!("npm install failed in frontend/. Fix errors above and retry."),
            Err(_) => anyhow::bail!("Node.js >=20 is required. Install from nodejs.org"),
        }
    }
    println!("{}", "[test] Running frontend tests (vitest)...".cyan());
    let vitest_status = Command::new("npm")
        .args(["run", "test"])
        .current_dir("frontend")
        .status()
        .await;
    match vitest_status {
        Ok(status) if !status.success() => anyhow::bail!("Frontend tests failed."),
        Err(_) => anyhow::bail!("Node.js >=20 is required. Install from nodejs.org"),
        Ok(_) => println!("{}", "[test] Frontend tests passed.".green()),
    }

    println!("{}", "All tests passed.".green().bold());
    Ok(())
}

async fn run_check() -> Result<()> {
    use tokio::process::Command;

    println!("{}", "[check] Type-checking Clarity contracts...".cyan());

    let status = Command::new("clarinet")
        .args(["check"])
        .current_dir("contracts")
        .status()
        .await;

    match status {
        Ok(s) if s.success() => {
            println!("{}", "[check] All contracts passed type-checking.".green());
            Ok(())
        }
        Ok(_) => {
            anyhow::bail!("Clarity type-check failed. Fix the errors reported above.");
        }
        Err(_) => {
            anyhow::bail!(
                "clarinet is required. Install: brew install clarinet OR cargo install clarinet"
            );
        }
    }
}

async fn run_clean() -> Result<()> {
    use std::path::Path;
    use tokio::fs;

    println!("{}", "[clean] Removing generated files and devnet state...".cyan());

    // Remove generated TypeScript bindings
    let generated_dir = Path::new("frontend/src/generated");
    if generated_dir.exists() {
        fs::remove_dir_all(generated_dir).await?;
        println!("{}", "[clean] Removed frontend/src/generated/".yellow());
    }

    // Remove Clarinet devnet state / cache
    let devnet_dir = Path::new("contracts/.cache");
    if devnet_dir.exists() {
        fs::remove_dir_all(devnet_dir).await?;
        println!("{}", "[clean] Removed contracts/.cache/".yellow());
    }

    // Remove devnet chain data if present
    let devnet_data = Path::new("contracts/.devnet");
    if devnet_data.exists() {
        fs::remove_dir_all(devnet_data).await?;
        println!("{}", "[clean] Removed contracts/.devnet/".yellow());
    }

    // Recreate the generated directory as empty (Next.js may expect it)
    fs::create_dir_all(generated_dir).await?;

    // Write an empty deployments.json so imports don't break
    fs::write(
        generated_dir.join("deployments.json"),
        r#"{ "network": "", "deployed_at": "", "contracts": {} }"#,
    )
    .await?;

    println!("{}", "[clean] Done. Run `stacks-dapp generate` to regenerate bindings.".green());
    Ok(())
}