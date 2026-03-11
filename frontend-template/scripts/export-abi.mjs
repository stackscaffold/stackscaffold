/**
 * Export Clarity contract ABIs from Clarinet.toml via @hirosystems/clarinet-sdk.
 * Outputs a JSON array of ContractAbi to stdout for stacks-dapp generate.
 * Run from frontend/: node scripts/export-abi.mjs
 */
import { initSimnet } from '@hirosystems/clarinet-sdk';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));

// When run from frontend/, contracts manifest is ../contracts/Clarinet.toml
const manifestPath = path.resolve(__dirname, '..', '..', 'contracts', 'Clarinet.toml');

const simnet = await initSimnet(manifestPath);
const raw = simnet.getContractsInterfaces();
const entries = raw instanceof Map ? [...raw.entries()] : Object.entries(raw || {});

const abis = [];
for (const [contractId, iface] of entries) {
  const name = contractId.includes('.') ? contractId.split('.').pop() : contractId;
  const fns = iface.functions || iface.functions_list || [];
  const functions = fns.map((fn) => ({
    name: fn.name,
    access: normalizeAccess(fn.access),
    args: fn.args || [],
    outputs: fn.outputs ?? fn.output ?? 'none',
  }));
  abis.push({
    contract_id: contractId,
    contract_name: name,
    functions,
    variables: iface.variables || [],
    maps: iface.maps || [],
    fungible_tokens: iface.fungible_tokens || [],
    non_fungible_tokens: iface.non_fungible_tokens || [],
  });
}

function normalizeAccess(access) {
  if (access == null) return 'private';
  const s = String(access).toLowerCase().replace(/-/g, '_');
  if (s === 'read_only' || s === 'readonly') return 'read_only';
  if (s === 'public' || s === 'private') return s;
  return s;
}

console.log(JSON.stringify(abis));
