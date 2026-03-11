import { StacksTestnet, StacksDevnet, StacksMainnet } from '@stacks/network';

const network = process.env.NEXT_PUBLIC_NETWORK ?? 'devnet';

export const scaffoldConfig = {
  targetNetwork: network,
  stacksNetwork:
    network === 'mainnet' ? new StacksMainnet()
    : network === 'testnet' ? new StacksTestnet()
    : new StacksDevnet(),
} as const;

