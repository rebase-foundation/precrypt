import '../styles/globals.css';
import '../styles/wallet-adapter.css';
import { WalletModalProvider } from '@solana/wallet-adapter-react-ui';
import type { AppProps } from 'next/app';
import dynamic from 'next/dynamic';
import { FC, ReactNode } from 'react';

// Use require instead of import, and order matters
require('../styles/globals.css');
require('@solana/wallet-adapter-react-ui/styles.css');

const WalletConnectionProvider = dynamic<{ children: ReactNode }>(
  () =>
    import('../components/WalletConnectionProvider').then(
      ({ WalletConnectionProvider }) => WalletConnectionProvider
    ),
  {
    ssr: false,
  }
);

const MyApp: FC<AppProps> = ({ Component, pageProps }) => {
  return (
    <div className="h-full flex flex-col">
      <WalletConnectionProvider>
        <WalletModalProvider>
          <Component {...pageProps} />
        </WalletModalProvider>
      </WalletConnectionProvider>
    </div>
  );
};
export default MyApp;
