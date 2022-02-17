import { useWallet } from '@solana/wallet-adapter-react';
import { WalletMultiButton } from '@solana/wallet-adapter-react-ui';
import type { NextPage } from 'next';
import loadConfig from 'next/dist/server/config';
import Link from 'next/link';
import { useState } from 'react';
import { useLocalStorage } from '../lib/useLocalStorage';

const Home: NextPage = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [resultDiv, setResultDiv] = useState((<div></div>));
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

  // Store File Params
  const [uploadFile, setUploadFile] = useState(null);

  async function onStoreKey() {
    setIsLoading(true);
    setResultDiv((<div></div>));
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
      setResultDiv((
        <div className='my-5 border border-black p-2 rounded bg-green-200'>
          <p className='font-bold'>Success</p>
          <p>
            Successfully stored key at CID: <a target={"_blank"} href={"https:/ipfs.io/ipfs/" + cid} className='text-blue-500 underline'>{cid}</a>
          </p>
        </div>
      ));
      setKeyCID(keyCID);
    } catch (error) {
      setResultDiv((
        <div className='my-5 border border-black p-2 rounded bg-red-200'>
          <p className='font-bold'>Failure</p>
          <p>
            Error storing key: {error}
          </p>
        </div>
      ));
    }
    setIsLoading(false);
  }

  async function onRequestKey() {
    setDecryptKey('');
    if (!keyCID || !publicKey || !signMessage) return;
    setIsLoading(true);

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
      setResultDiv((
        <div className='my-5 border border-black p-2 rounded bg-green-200'>
          <p className='font-bold'>Success</p>
          <p>
            Successfully retrieved key with CID: <a target={"_blank"} href={"https:/ipfs.io/ipfs/" + keyCID} className='text-blue-500 underline'>{keyCID}</a>. 
          </p>
          <p>
            You can now decrypt file at CID: <a target={"_blank"} href={"https:/ipfs.io/ipfs/" + fileCID} className='text-blue-500 underline'>{fileCID}</a> with extension: {json['file_extension']}
          </p>
        </div>
      ));
    } catch (error) {
      console.log(error);
      setResultDiv((
        <div className='my-5 border border-black p-2 rounded bg-red-200'>
          <p className='font-bold'>Failure</p>
          <p>
            Error storing key: {error}
          </p>
        </div>
      ));
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

  async function onStoreFile() {
    if (!mintAddress || !uploadFile) return;
    setIsLoading(true);
    const formData = new FormData;
    formData.append('mint', mintAddress);
    formData.append('', uploadFile as Blob);
    console.log(formData);
    try {
      const resp = await fetch(`${proxyEndpoint}/file/store`, {
        method: 'POST',
        body: formData
      });
      const json = await resp.json();
      console.log(json);
    } catch (error) {
      console.log(error);
    }
    setIsLoading(false);
  }

  async function onRequestFile() {
    if (!keyCID || !publicKey || !signMessage) return;
    setIsLoading(true);

    // Create signature with browser wallet
    const message = new TextEncoder().encode('precrypt');
    const signature = await signMessage(message);

    let body = JSON.stringify({
      'key_cid': keyCID,
      'sol_pubkey': Array.from(publicKey.toBytes()),
      'sol_signed_message': Array.from(signature)
    });
    console.log(body);
    try {
      const resp = await fetch(`${proxyEndpoint}/file/request`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: body
      });
      const json = await resp.json();
      console.log(json);
    } catch (error) {
      console.log(error);
    }
    setIsLoading(false);
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

        {/* KEY NOTICE */}
        {requestType === 'key' && <div className='my-5 border border-black p-2 rounded bg-green-200'>
          <p className='font-bold'>Using the CLI</p>
          <p>
            To locally encrypt your file and generate keys, you will need the Precrypt CLI. You can go to the <a target={"_blank"} href="https://crates.io/crates/precrypt" className='text-blue-500 underline'>
              Precrypt crates.io</a> page for instructions on how to install and use the CLI.
          </p>
        </div>}

        {/* FILE NOTICE */}
        {requestType === 'file' && <div className='my-5 border border-black p-2 rounded bg-red-200'>
          <p className='font-bold'>Warning about uploading files</p>
          <p>
            A primary advantage of proxy based re-encryption is that the proxy does not need to see files. Precrypt offers a file based flow for ease of use, but it is <b>less secure and reliable. </b>
            <a className='text-blue-500 underline' target={"_blank"} href='https://precrypt.org'>Learn more</a>
          </p>
        </div>}

        {/* INPUTS */}
        {requestType === 'key' && <div className='flex flex-row gap-2'>
          <div className='w-1/2 border-2 rounded p-2'>
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
          <div className='w-1/2 border-2 rounded p-2'>
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
        </div>}

        {requestType === 'file' && <div className='flex flex-row gap-2'>
          <div className='w-1/2 border-2 rounded p-2'>
            <div className='border-b-2 text-xl text-center font-bold'>Store</div>
            Plaintext Key: <input
              type={'file'}
              onChange={async (e: any) => {
                const file = e.target.files[0];
                if (!file) {
                  setUploadFile(null);
                  return
                }
                setUploadFile(file);
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
            <button onClick={onStoreFile} disabled={isLoading || !mintAddress || !uploadFile} className='border border-black rounded bg-gray-300 px-2 mx-auto disabled:opacity-20'>Submit</button>
          </div>
          <div className='w-1/2 border-2 rounded p-2'>
            <div className='border-b-2 text-xl text-center font-bold'>Request</div>
            <WalletMultiButton />
            <label>
              Key CID:
              <input
                className='border ml-2'
                type={'text'}
                onChange={(e) => setKeyCID(e.target.value)}
                value={keyCID}
              />
            </label>
            <br></br>
            <button onClick={onRequestFile} disabled={isLoading || !keyCID} className='border border-black rounded bg-gray-300 px-2 mx-auto disabled:opacity-20'>Submit</button>
          </div>
        </div>}
        {resultDiv && <div>
          {resultDiv}
        </div>}
        
        {decryptKey && <button onClick={onDownloadKey} className='border border-black rounded bg-gray-300 px-2 mt-3 mx-auto'>Download Decryption Key</button>}
      </div>
    </div>
  );
};

export default Home;
