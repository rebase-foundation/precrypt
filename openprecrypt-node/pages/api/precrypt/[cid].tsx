import { Program, Provider, web3 } from '@project-serum/anchor';
import { Transaction, Keypair, clusterApiUrl, Cluster } from '@solana/web3.js';
import { NextApiRequest, NextApiResponse } from 'next';
import { Web3Storage, File } from 'web3.storage';
import initMiddleware from '../../../lib/initMiddleware';
import Cors from 'cors';
import { decrypt } from '../../../lib/nacl';
import * as umbral from "umbral-pre";

let enc = new TextEncoder();
let dec = new TextDecoder("utf-8");

// Initialize the cors middleware
const cors = initMiddleware(
   // You can read more about the available options here: https://github.com/expressjs/cors#configuration-options
   Cors({
      // Only allow requests with GET, POST and OPTIONS
      methods: ['GET', 'POST', 'OPTIONS'],
      origin: '*',
   })
);

/**
 * 
 */
export default async function handler(
   req: NextApiRequest,
   res: NextApiResponse
) {
   // Run cors
   await cors(req, res);

   try {
      switch (req.method) {
         case 'GET':
            // Get the data from IPFS
            const cid = req.query['cid'] as string;
            if (!cid) { res.status(400).end("CID not found in request") }

            const web3Client = new Web3Storage({ token: process.env.WEB_3_STORAGE_TOKEN! });
            const web3_res = await web3Client.get(cid);
            if (!web3_res || !web3_res.ok) {
               throw new Error(`failed to get ${cid}`)
            }
            const blobs = await web3_res.files(); // We don't use directory wrapping so there should only be 1 file
            const blob = blobs.pop();
            const cipher = await blob!.text();
            
            // Decrypt data with sever private key
            const key = process.env.NACL_SECRET!;
            const json = decrypt(cipher, key);

            const mint = json['mint'];
            const recrypt_key = JSON.parse(json['recrypt_key']);

            // TODO: Verify that the getter holds a token from the mind

            // Translate the recryption key into a decryption key

            // TODO: Get the actual delegate private
            const delegate_pubkey = umbral.SecretKey.random().publicKey();

            // const capsule_bytes: Uint8Array = recrypt_key["capsules"][0]
            // console.log();
            // const capsule = umbral.Capsule.fromBytes(Buffer.from(capsule_bytes));
            // console.log(capsule);
            // const capsule_u8 = Buffer.from(capsule, "base64").toString('utf8');
            // const capsule_bytes = Buffer.from(capsule_u8, "utf8");
            // console.log(capsule_bytes);
            

            const owner_secret = umbral.SecretKey.fromBytes(recrypt_key["owner_secret"]);
            console.log(owner_secret);
            const translation_key = umbral.generateKFrags(
               owner_secret,
               delegate_pubkey,
               new umbral.Signer(umbral.SecretKey.random()),
               1,
               1,
               false,
               false
            )[0];
            console.log(translation_key);
            
            for (const c of recrypt_key["capsules"]) {
               // console.log(enc.encode(c));
               console.log(c.length);
               const capsule_frag =  umbral.Capsule.fromBytes(c);
            }
            // const capsule_ser = recrypt_key['capsules'][0];
            // console.log(capsule_ser);
            // console.log(enc.encode(capsule_ser));
            // console.log(capsule_frag);

            const translated_keys = [];

            // console.log(translation_key)
            return res.end(JSON.stringify(recrypt_key));
         default:
            res.setHeader('Allow', ['GET'])
            return res.status(405).end(`Method ${req.method} Not Allowed`)
      }
   } catch (err) {
      console.error(err);
      return res.status(500).send('Something went wrong');
   }
}

// export const config = {
//   api: {
//     // bodyParser: false
//       bodyParser: {
//           sizeLimit: '10mb' // Set desired value here

//       }
//   }
// }



// RPC QUERY TO SEE IF WALLET HOLDS TOKEN OF MINT
// https://ssc-dao.genesysgo.net/
// {
//    "jsonrpc": "2.0",
//    "id": 1,
//    "method": "getTokenAccountsByOwner",
//    "params": [
//      "J2oTMRXsh59yH9P5Xpoz7FGreXa1mKxf4wi1uDsrx8HR",
//      {
//        "mint": "moodn6VC7wWoFEmx5xGRkFJTNqXdiWBE2c9a3JhEC5p"
//      },
//      {
//        "encoding": "jsonParsed"
//      }
//    ]
//  }\