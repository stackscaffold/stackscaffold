"use client";
import { createContext, useContext, useState, ReactNode } from 'react';
import { showConnect, type FinishedAuthData } from '@stacks/connect';
import { scaffoldConfig } from '@/scaffold.config';

type WalletContextValue = {
  address: string | null;
  connected: boolean;
  setAddress: (addr: string | null) => void;
  disconnect: () => void;
};

const WalletContext = createContext<WalletContextValue | undefined>(undefined);

export function WalletProvider({ children }: { children: ReactNode }) {
  const [address, setAddress] = useState<string | null>(null);

  return (
    <WalletContext.Provider value={{
      address,
      connected: !!address,
      setAddress,
      disconnect: () => setAddress(null),
    }}>
      {children}
    </WalletContext.Provider>
  );
}

export function useWallet() {
  const ctx = useContext(WalletContext);
  if (!ctx) throw new Error('useWallet must be used within WalletProvider');
  return ctx;
}

export function WalletConnect() {
  const { address, connected, setAddress, disconnect } = useWallet();

  const onConnect = () => {
    showConnect({
      appDetails: {
        name: 'scaffold-stacks',
        icon: '/favicon.ico',
      },
      network: scaffoldConfig.stacksNetwork,
      onFinish: (data: FinishedAuthData) => {
        const profile = data.userSession?.loadUserData()?.profile;
        const addr = scaffoldConfig.isMainnet
          ? profile?.stxAddress?.mainnet
          : profile?.stxAddress?.testnet;
        if (addr) setAddress(addr);
      },
      onCancel: () => {},
    });
  };

  if (!connected) {
    return (
      <button
        className="px-3 py-1.5 rounded bg-emerald-500 hover:bg-emerald-400 text-black text-sm font-semibold transition-colors"
        onClick={onConnect}
      >
        Connect Wallet
      </button>
    );
  }

  const short = address ? `${address.slice(0, 6)}…${address.slice(-4)}` : '';

  return (
    <div className="flex items-center gap-2 text-sm">
      <span className="px-2 py-1 rounded bg-gray-800 font-mono text-emerald-400">
        {short}
      </span>
      <button
        className="px-2 py-1 rounded bg-gray-700 hover:bg-gray-600 text-xs transition-colors"
        onClick={disconnect}
      >
        Disconnect
      </button>
    </div>
  );
}