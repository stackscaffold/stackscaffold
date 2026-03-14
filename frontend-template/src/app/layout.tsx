import type { ReactNode } from 'react';
import './globals.css';
import { WalletProvider } from '../components/WalletConnect';

export const metadata = {
  title: 'scaffold-stacks',
  description: 'Built with scaffold-stacks',
};

export default function RootLayout({ children }: { children: ReactNode }) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-gray-950 text-white">
        <WalletProvider>
          {children}
        </WalletProvider>
      </body>
    </html>
  );
}