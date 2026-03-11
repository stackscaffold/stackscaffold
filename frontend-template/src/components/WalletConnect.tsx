\"use client\";
import { createContext, useContext, useState, ReactNode } from 'react';
import { showConnect } from '@stacks/connect';

type WalletContextValue = {
  address: string | null;
  connected: boolean;
  disconnect: () => void;
};

const WalletContext = createContext<WalletContextValue | undefined>(undefined);

export function WalletProvider({ children }: { children: ReactNode }) {
  const [address, setAddress] = useState<string | null>(null);

  const disconnect = () => setAddress(null);

  const value: WalletContextValue = {
    address,
    connected: !!address,
    disconnect,
  };

  return <WalletContext.Provider value={value}>{children}</WalletContext.Provider>;
}

export function useWallet(): WalletContextValue {
  const ctx = useContext(WalletContext);
  if (!ctx) {
    throw new Error('useWallet must be used within WalletProvider');
  }
  return ctx;
}

export function WalletConnect() {
  const { address, connected, disconnect } = useWallet();

  const onConnect = () => {
    showConnect({
      onFinish: data => {
        // @ts-ignore
        const addr = data.address || data.profile?.stxAddress?.mainnet || null;
        if (addr) {
          // eslint-disable-next-line no-console
          console.log('Connected', addr);
        }
      },
    });
  };

  if (!connected) {
    return (
      <button
        className=\"px-3 py-1 rounded bg-emerald-500 text-black text-sm\"
        onClick={onConnect}
      >
        Connect Wallet
      </button>
    );
  }

  const short = address ? `${address.slice(0, 6)}…${address.slice(-4)}` : '';

  return (
    <div className=\"flex items-center gap-2 text-sm\">
      <span className=\"px-2 py-1 rounded bg-gray-800 font-mono\">{short}</span>
      <button
        className=\"px-2 py-1 rounded bg-gray-700 text-xs\"
        onClick={disconnect}
      >
        Disconnect
      </button>
    </div>
  );
}

