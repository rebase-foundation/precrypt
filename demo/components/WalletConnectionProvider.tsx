import { WalletAdapterNetwork } from '@solana/wallet-adapter-base';
import {
  ConnectionProvider,
  WalletProvider,
} from '@solana/wallet-adapter-react';
import {
  getLedgerWallet,
  getPhantomWallet,
  getSlopeWallet,
  getSolflareWallet,
  getSolletExtensionWallet,
  getSolletWallet,
  getTorusWallet,
} from '@solana/wallet-adapter-wallets';
import { clusterApiUrl } from '@solana/web3.js';
import { FC, ReactNode, useMemo } from 'react';
import { useFlag } from '../lib/useFlag';

let networks = {
  'mainnet-beta': WalletAdapterNetwork.Mainnet,
  testnet: WalletAdapterNetwork.Testnet,
};

export const WalletConnectionProvider: FC<{ children: ReactNode }> = ({
  children,
}) => {
  const flag = useFlag('network', 'mainnet-beta');
  const network = networks[flag];

  // You can also provide a custom RPC endpoint
  const endpoint =
    flag == 'mainnet-beta'
      ? 'https://ssc-dao.genesysgo.net/'
      : "'https://rpc-testnet.rebasefoundation.org/'";

  // @solana/wallet-adapter-wallets includes all the adapters but supports tree shaking --
  // Only the wallets you configure here will be compiled into your application
  const wallets = useMemo(
    () => [
      getPhantomWallet(),
      getSlopeWallet(),
      getSolflareWallet(),
      //   getTorusWallet({
      //     options: { clientId: 'Get a client ID @ https://developer.tor.us' },
      //   }),
      getLedgerWallet(),
      getSolletWallet({ network }),
      getSolletExtensionWallet({ network }),
    ],
    [network]
  );

  return (
    <ConnectionProvider endpoint={endpoint}>
      <WalletProvider wallets={wallets} autoConnect>
        {children}
      </WalletProvider>
    </ConnectionProvider>
  );
};
