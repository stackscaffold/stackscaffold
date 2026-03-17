

stacks scaffold is a Rust-powered CLI (`stacksdapp`) and Next.js template that lets you scaffold, develop, and deploy full-stack Stacks (Bitcoin L2) dApps with auto-generated contract bindings and a live debug UI.

---

## Prerequisites

Install these before anything else.

| Tool | Install | Required for |
|---|---|---|
| **Rust** (stable 1.75+) | [rustup.rs](https://rustup.rs) | Building the CLI |
| **Node.js** 20+ | [nodejs.org](https://nodejs.org) | Frontend + contract tests |
| **Clarinet** | `brew install clarinet` | Contract toolchain |
| **Docker Desktop** | [docker.com](https://docker.com) | Devnet only |

Verify everything is ready:

```bash
rustc --version      # rustc 1.75+
node --version       # v20+
clarinet --version   # clarinet 3.x
docker info          # must be running for devnet
```

---

## Install the CLI

```bash
# Clone the scaffold-stacks repo
git clone https://github.com/YOUR_ORG/scaffold-stacks
cd scaffold-stacks

# Build and install the CLI binary to your PATH
cargo install --path cli

# Confirm it works
stacksdapp --version
```

---

## Developer Guide — From Zero to Running dApp

### Step 1 — Scaffold a new project

```bash
stacksdapp new my-app
```

This creates the following structure:

```
my-app/
├── contracts/                  # Clarity smart contracts
│   ├── Clarinet.toml
│   ├── settings/
│   │   ├── Devnet.toml         # pre-funded local accounts
│   │   ├── Testnet.toml        # add your mnemonic here for testnet
│   │   └── Mainnet.toml        # add your mnemonic here for mainnet
│   ├── contracts/
│   │   └── counter.clar        # sample contract
│   └── tests/
│       └── counter.test.ts     # sample Vitest tests
├── frontend/
│   ├── .env.local              # NEXT_PUBLIC_NETWORK=devnet (auto-managed)
│   ├── src/
│   │   ├── app/
│   │   │   ├── layout.tsx
│   │   │   ├── page.tsx
│   │   │   └── globals.css
│   │   ├── components/
│   │   │   ├── WalletConnect.tsx
│   │   │   └── debug/
│   │   │       └── DebugContracts.tsx
│   │   ├── generated/          # ← auto-generated, never edit by hand
│   │   │   ├── contracts.ts
│   │   │   ├── hooks.ts
│   │   │   ├── DebugContracts.tsx
│   │   │   └── deployments.json
│   │   └── scaffold.config.ts
│   └── scripts/
│       └── export-abi.mjs
└── package.json
```

### Step 2 — Start the dev environment

```bash
cd my-app
stacksdapp dev
```

This does four things concurrently:

1. Runs `stacksdapp generate` to build initial TypeScript bindings
2. Starts `clarinet devnet start` — a local Bitcoin + Stacks node via Docker
3. Starts `next dev` — the frontend on [http://localhost:3000](http://localhost:3000)
4. Watches `contracts/contracts/` — reruns generate on every `.clar` save

Open [http://localhost:3000](http://localhost:3000). You'll see the Debug Contracts panel with buttons for every function in `counter.clar`.

> **Docker must be running** before `stacksdapp dev`. If you just want to build the frontend without a local chain, skip to the testnet workflow below.

### Step 3 — Deploy contracts to devnet

In a second terminal, once `stacksdapp dev` is running and the node is ready (~30s):

```bash
cd my-app
stacksdapp deploy --network devnet
```

Output:
```
🚀 Deploying to devnet (http://localhost:3999)
[deploy] Waiting for Stacks node...
[deploy] ✔ Node is ready
[deploy] Generating deployment plan...
[deploy] Applying deployment plan to devnet...
  ✔ counter | txid 0x86fa3030... | address ST1PQHQ....counter
[deploy] Written to frontend/src/generated/deployments.json
```

The frontend automatically picks up the new contract addresses from `deployments.json` and the Debug Contracts panel becomes fully interactive.

### Step 4 — Edit contracts and see live updates

Open `contracts/contracts/counter.clar` and add a new function:

```clarity
(define-public (multiply (n uint))
  (begin
    (var-set counter (* (var-get counter) n))
    (ok (var-get counter))))
```

Save the file. The file watcher detects the change, reruns `stacksdapp generate`, and Next.js hot-reloads. The `multiply` button appears in the debug panel automatically — no manual steps needed.

### Step 5 — Add a new contract

```bash
stacksdapp add relayer
# Creates contracts/contracts/relayer.clar
# Updates Clarinet.toml
# Regenerates all TypeScript bindings immediately
```

Use a template:
```bash
stacksdapp add token --template sip010    # fungible token
stacksdapp add nft   --template sip009    # NFT
```

### Step 6 — Manually regenerate bindings

If you ever need to force a regeneration outside of the file watcher:

```bash
stacksdapp generate
```

Output:
```
[generate] Parsing contract ABIs...
[generate] Found 2 contract(s): counter, relayer
[generated] frontend/src/generated/contracts.ts
[generated] frontend/src/generated/hooks.ts
[generated] frontend/src/generated/DebugContracts.tsx
[generate] Done — 3 file(s) written.
```

If files are already up to date, you'll see:
```
[generate] All files already up to date.
```

### Step 7 — Run tests

```bash
stacksdapp test
# Runs vitest in contracts/  (Clarinet SDK, no Docker needed)
# Runs vitest in frontend/
```

Run just contract tests directly:
```bash
cd contracts && npm test
```

### Step 8 — Type-check contracts

```bash
stacksdapp check
# Runs clarinet check in contracts/
```

### Step 9 — Clean up

```bash
stacksdapp clean
# Removes frontend/src/generated/
# Removes contracts/.cache/ and contracts/.devnet/
# Recreates empty deployments.json
```

---

## Testnet Workflow

No Docker needed — contracts run on the Hiro testnet infrastructure.

```bash
# 1. Get testnet STX from the faucet
#    https://explorer.hiro.so/sandbox/faucet?chain=testnet

# 2. Add your deployer mnemonic to contracts/settings/Testnet.toml
#    [accounts.deployer]
#    mnemonic = "your 24 words here"

# 3. Deploy contracts to testnet
stacksdapp deploy --network testnet

# 4. Start the frontend pointed at testnet
stacksdapp dev --network testnet
# → opens http://localhost:3000 with NEXT_PUBLIC_NETWORK=testnet
# → Connect your Leather/Xverse wallet set to Testnet
```

---

## Mainnet Workflow

```bash
# 1. Complete thorough testing on testnet first
# 2. Add your deployer mnemonic to contracts/settings/Mainnet.toml
# 3. Ensure you have enough STX for fees

stacksdapp deploy --network mainnet
stacksdapp dev --network mainnet
```

---

## Command Reference

| Command | Description |
|---|---|
| `stacksdapp new <name>` | Scaffold a new monorepo with contracts + frontend |
| `stacksdapp dev` | Start devnet + Next.js + file watcher (Docker required) |
| `stacksdapp dev --network testnet` | Run frontend against testnet (no Docker) |
| `stacksdapp dev --network mainnet` | Run frontend against mainnet (no Docker) |
| `stacksdapp generate` | Parse contract ABIs → regenerate TS bindings + debug UI |
| `stacksdapp add <name>` | Add a new Clarity contract (blank template) |
| `stacksdapp add <name> --template sip010` | Add a SIP-010 fungible token contract |
| `stacksdapp add <name> --template sip009` | Add a SIP-009 NFT contract |
| `stacksdapp deploy --network devnet` | Deploy to local devnet |
| `stacksdapp deploy --network testnet` | Deploy to testnet |
| `stacksdapp deploy --network mainnet` | Deploy to mainnet |
| `stacksdapp test` | Run contract tests (Vitest + Clarinet SDK) and frontend tests |
| `stacksdapp check` | Type-check all Clarity contracts via Clarinet |
| `stacksdapp clean` | Remove generated files and devnet state |

---

## How Auto-Codegen Works

`stacksdapp generate` runs in four stages:

1. **Parse** — runs `export-abi.mjs` with `CWD=contracts/` which calls `initSimnet()` to extract the ABI of every contract listed in `Clarinet.toml`
2. **Normalise** — maps Clarity types to TypeScript (`uint128` → `bigint`, `string-ascii` → `string`, `tuple` → `{ field: T }`, etc.)
3. **Render** — feeds the normalised ABI into three Tera templates to produce `contracts.ts`, `hooks.ts`, and `DebugContracts.tsx`
4. **Write** — compares SHA-256 hashes of new vs existing files; only writes if content changed, keeping Next.js hot-reload fast

The file watcher calls this pipeline automatically on every `.clar` save during `stacksdapp dev`.

---

## Project Structure (Rust crates)

```
cli/                    # Binary crate — clap CLI entrypoint
crates/
  scaffold/             # stacksdapp new + stacksdapp add
  parser/               # Clarity ABI → Rust structs
  codegen/              # Rust structs → TypeScript via Tera templates
  watcher/              # notify-based file watcher
  deployer/             # clarinet deployments generate + apply
  process_supervisor/   # orchestrates dev command per network
templates/
  contracts.ts.tera     # typed contract call wrappers
  hooks.ts.tera         # React hooks
  debug_ui.tsx.tera     # live debug panel
frontend-template/      # copied into every new project
```

---

## Contributing

```bash
git clone https://github.com/stackscaffold/stackscaffold.git
cd stackscaffold
cargo build
cargo test --all
```

Open issues or pull requests for bugs, features, or DX improvements. Each crate is focused and independently testable.

---

## License

MIT