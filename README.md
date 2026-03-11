![Build](https://img.shields.io/github/actions/workflow/status/YOUR_ORG/scaffold-stacks/ci.yml?branch=main)
![Crates.io](https://img.shields.io/crates/v/stacks-dapp)
![License](https://img.shields.io/badge/license-MIT-blue.svg)

scaffold-stacks is a Rust-powered CLI (`stacks-dapp`) and Next.js template that lets you scaffold, develop, and deploy full-stack Stacks (Bitcoin L2) dApps with auto-generated contract bindings and a live debug UI.

Prerequisites
-------------

- **Rust** (stable, 1.75+ recommended) via `rustup`
- **Node.js** 20+
- **Clarinet** (`brew install clarinet` or `cargo install clarinet`)

Quickstart
----------

```bash
# 1. Clone and build the toolkit
git clone https://github.com/YOUR_ORG/scaffold-stacks
cd scaffold-stacks
cargo build --release

# 2. Install the CLI binary locally
cargo install --path cli

# 3. Scaffold a new Stacks dApp
stacks-dapp new my-app
cd my-app

# 4. Start local devnet + frontend
stacks-dapp dev
```

Command Reference
-----------------

| Command                          | Description                                                   |
| -------------------------------- | ------------------------------------------------------------- |
| `stacks-dapp new <name>`         | Scaffold a new monorepo with contracts + frontend            |
| `stacks-dapp dev`                | Run Clarinet devnet, Next.js dev server, and ABI watcher     |
| `stacks-dapp generate`           | Parse contract ABIs and regenerate TS bindings + debug UI    |
| `stacks-dapp add contract <name>`| Add a new Clarity contract and update configuration          |
| `stacks-dapp deploy --network`   | Deploy contracts and write `deployments.json` for frontend   |
| `stacks-dapp test`               | Run contract and frontend tests (Clarinet + Vitest)          |
| `stacks-dapp check`              | Type-check all Clarity contracts via Clarinet                |
| `stacks-dapp clean`              | Remove generated files and devnet state                      |

How Auto-Codegen Works
----------------------

`stacks-dapp generate` calls into the Rust `parser` and `codegen` crates to read Clarinet-emitted ABI JSON, normalise Clarity types to TypeScript, and render three outputs via Tera templates: typed contract wrappers (`contracts.ts`), React hooks (`hooks.ts`), and a live `DebugContracts` panel. These files land in `frontend/src/generated/` and are overwritten only when the generated hash changes, so your frontend hot-reload stays fast while always reflecting the latest contract surface.

Contributing
------------

Clone the repo, ensure the prerequisites above are installed, then:

```bash
cargo build
cargo test --all
```

Open issues or pull requests for bugs, features, or DX improvements. The architecture is split into focused crates (`cli`, `scaffold`, `parser`, `codegen`, `watcher`, `deployer`, `process_supervisor`) to make contributions easy to reason about.

License
-------

MIT


