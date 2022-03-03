import '../styles/globals.css';
import type { AppProps } from 'next/app';
import Head from "next/head";
import { FC } from 'react';

const MyApp: FC<AppProps> = ({ Component, pageProps }) => {
  return (
    <>
      <Head>
        <title>Precrypt | Premissioned files on IPFS</title>
      </Head>
      <div className="h-full flex flex-col">
        <Component {...pageProps} />
      </div>
    </>
  );
};
export default MyApp;
