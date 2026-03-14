import { StacksTestnet, StacksDevnet, StacksMainnet } from '@stacks/network';

const network = process.env.NEXT_PUBLIC_NETWORK ?? 'devnet';

const nodeUrls: Record<string, { stacks: string; bitcoin: string }> = {
  devnet: {
    stacks:  process.env.NEXT_PUBLIC_STACKS_NODE_URL ?? 'http://localhost:3999',
    bitcoin: 'http://localhost:18443',
  },
  testnet: {
    stacks:  process.env.NEXT_PUBLIC_STACKS_NODE_URL ?? 'https://api.testnet.hiro.so',
    bitcoin: 'https://blockstream.info/testnet/api',
  },
  mainnet: {
    stacks:  process.env.NEXT_PUBLIC_STACKS_NODE_URL ?? 'https://api.hiro.so',
    bitcoin: 'https://blockstream.info/api',
  },
};

export const scaffoldConfig = {
  targetNetwork: network,

  stacksNetwork:
    network === 'mainnet' ? new StacksMainnet()
    : network === 'testnet' ? new StacksTestnet()
    : new StacksDevnet(),

  nodeUrl: nodeUrls[network] ?? nodeUrls.devnet,

  // Convenience flags
  isDevnet:  network === 'devnet',
  isTestnet: network === 'testnet',
  isMainnet: network === 'mainnet',
} as const;