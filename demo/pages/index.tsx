import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import type { NextPage } from 'next';
import loadConfig from 'next/dist/server/config';
import Link from 'next/link';
import { useState } from 'react';
import { useLocalStorage } from '../lib/useLocalStorage';

const Home: NextPage = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [resultMessage, setResultMessage] = useLocalStorage('result', '');
  const [proxyEndpoint, setProxyEndpoint] = useLocalStorage('endpoint', 'https://precrypt.org');
  const [requestType, setRequestType] = useLocalStorage('type', '');
  const { publicKey, signMessage } = useWallet();

  // Store Key Params
  const [recryptionKeyString, setRecryptionKeyString] = useLocalStorage('recryptKey', '');
  const [mintAddress, setMintAddress] = useLocalStorage('mint', '');
  const [fileCID, setFileCID] = useLocalStorage('fileCID', '');
  const [fileExtension, setFileExtension] = useLocalStorage('extension', '');

  // Request Key Params
  const [keyCID, setKeyCID] = useLocalStorage('keyCID', '');
  const [precryptPubkey, setPrecryptPubkey] = useLocalStorage('precryptPubkey', '');
  const [decryptKey, setDecryptKey] = useLocalStorage('decryptKey', '');

  async function onStoreKey() {
    setIsLoading(true);

    const body = JSON.stringify({
      "recryption_keys": JSON.parse(recryptionKeyString),
      "mint": mintAddress,
      "file_cid": fileCID,
      "file_extension": fileExtension
    });
    console.log(body);
    try {
      const resp = await fetch(`${proxyEndpoint}/key/store`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: body
      });
      const json = await resp.json();
      console.log(json);
      let cid = json['cid'];
      setResultMessage(`Successfully stored key at CID: ${cid}`);
    } catch (error) {
      console.log(error);
    }
    setIsLoading(false);
  }

  async function onRequestKey() {
    setDecryptKey('');
    setIsLoading(true);
    if (!keyCID || !publicKey || !signMessage) return;
    
    // Create signature with browser wallet
    const message = new TextEncoder().encode('precrypt');
    const signature = await signMessage(message);

    let body = JSON.stringify({
      'key_cid': keyCID,
      'precrypt_pubkey': JSON.parse(precryptPubkey),
      'sol_pubkey': Array.from(publicKey.toBytes()),
      'sol_signed_message': Array.from(signature)
    });
    console.log(body);
    try {
      const resp = await fetch(`${proxyEndpoint}/key/request`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: body
      });
      const json = await resp.json();
      console.log(json);
      setDecryptKey(json['decryption_keys']);
      setResultMessage(`Successfully retrieved key with CID: ${keyCID}. You can now decrypt file at CID: ${json['file_cid']} with extension: ${json['file_extension']}`);
    } catch (error) {
      console.log(error);
    }
    setIsLoading(false);
  }

  function onDownloadKey() {
    const blob = new Blob([JSON.stringify(decryptKey)], { type: 'text/json' });
    const elem = window.document.createElement('a');
    elem.href = window.URL.createObjectURL(blob);
    elem.download = "decrypt.json";
    document.body.appendChild(elem);
    elem.click();
    document.body.removeChild(elem);
 }

  return (
    <div>
      <div className='border-b'>
        <div className='px-4 py-4 mx-auto max-w-4xl flex flex-flex items-center justify-between'>
          <h1 className='font-bold text-center text-2xl m-0 p-0'>
            Precrypt Demo
          </h1>
          <div className='flex flex-row items-center'>
            <p className='pr-2'>
              Trusted Proxy:
            </p>
            <select className='border-2 rounded p-1' value={proxyEndpoint} onChange={(e) => setProxyEndpoint(e.target.value)}>
              <option value="https://precrypt.org">https://precrypt.org</option>
              <option value="http://localhost:8000">http://localhost:8000</option>
            </select>
            {/* <p className='text-4xl pl-2 pb-1 text-green-500'>•</p> */}
          </div>
        </div>
      </div>
      <div className='m-auto h-full px-4 py-4 max-w-2xl items-center text-left'>
        <div className='my-auto text-xl font-bold'>Chose your request type:</div>
        <div>
          <p className='py-2'>There are two ways to use precrypt. You can to encrypt/decrypt the file locally and have the proxy store/generate a <b>key</b>. You can also upload/request a <b>file</b> and have the encryption/decryption take place on the server.</p>
          <input type={"radio"} value="key" name='key' checked={requestType == 'key'} onChange={(e) => setRequestType(e.target.value)}></input> Key (recommended)
          <br></br>
          <input type={"radio"} value="file" name='file' checked={requestType == 'file'} onChange={(e) => setRequestType(e.target.value)}></input> File
        </div>
        <div className='flex flex-row gap-2 py-4'>
          <div className='w-1/2 border-2 p-2'>
            <div className='border-b-2 text-xl text-center font-bold'>Store</div>
            Recryption Key: <input
              type={'file'}
              onChange={async (e: any) => {
                const file = e.target.files[0];
                if (!file) {
                  setRecryptionKeyString('');
                  return
                }
                try {
                  var reader = new FileReader();
                  reader.addEventListener('load', function (e) {
                    if (!e.target) {
                      console.log("parse error")
                      return;
                    }
                    console.log(e.target.result);
                    setRecryptionKeyString(e.target.result as string);
                  });
                  reader.readAsBinaryString(file);
                } catch (error) {
                  console.log('Error parsing file: ', error);
                }
              }}
            >
            </input>
            <br></br>
            <label>
              Mint Address:
              <input
                className='border ml-2'
                type={'text'}
                onChange={(e) => setMintAddress(e.target.value)}
                value={mintAddress}
              />
            </label>
            <br></br>
            <label>
              File CID:
              <input
                className='border ml-2'
                type={'text'}
                onChange={(e) => setFileCID(e.target.value)}
                value={fileCID}
              />
            </label>
            <br></br>
            <label>
              File Extension:
              <input
                className='border ml-2'
                type={'text'}
                onChange={(e) => setFileExtension(e.target.value)}
                value={fileExtension}
              />
            </label>
            <br></br>
            <button onClick={onStoreKey} disabled={isLoading || !recryptionKeyString || !mintAddress || !fileCID || !fileExtension} className='border border-black rounded bg-gray-300 px-2 mx-auto disabled:opacity-20'>Submit</button>
          </div>
          <div className='w-1/2 border-2 p-2'>
            <div className='border-b-2 text-xl text-center font-bold'>Request</div>
            <WalletMultiButton />
            <br></br>
            <label>
              Pubkey:
              <input
                className='border ml-2'
                type={'text'}
                onChange={(e) => setPrecryptPubkey(e.target.value)}
                value={precryptPubkey}
              />
            </label>
            <br></br>
            <label>
              Key CID:
              <input
                className='border ml-2'
                type={'text'}
                onChange={(e) => setKeyCID(e.target.value)}
                value={keyCID}
              />
            </label>
            <button onClick={onRequestKey} disabled={isLoading || !precryptPubkey || !keyCID || !publicKey} className='border border-black rounded bg-gray-300 px-2 mx-auto disabled:opacity-20'>Submit</button>
          </div>
        </div>
        {resultMessage && <div>
          {resultMessage}
        </div>}
        {decryptKey && <button onClick={onDownloadKey} className='border border-black rounded bg-gray-300 px-2 mt-3 mx-auto'>Download Decryption Key</button>}
      </div>
    </div>
  );
};

export default Home;
