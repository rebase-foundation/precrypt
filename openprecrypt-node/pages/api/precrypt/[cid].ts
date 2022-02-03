import { Program, Provider, web3 } from '@project-serum/anchor';
import { Transaction, Keypair, clusterApiUrl, Cluster } from '@solana/web3.js';
import { NextApiRequest, NextApiResponse } from 'next';
import { Web3Storage, File } from 'web3.storage';
import initMiddleware from '../../../lib/initMiddleware';
import Cors from 'cors';

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
            const web3Client = new Web3Storage({ token: process.env.WEB_3_STORAGE_TOKEN });
            const web3_res = await web3Client.get(cid);
            if (!web3_res.ok) {
               throw new Error(`failed to get ${cid}`)
            }
            const blobs = await web3_res.files(); // We don't use directory wrapping so there should only be 1 file
            const blob = blobs.pop();
            
            // TODO: Decrypt data with sever private key
            
            const text = await blob.text();
            const json = JSON.parse(text);
            const mint = json['mint'];
            const recrypt_key = json['recrypt_key'];

            // TODO: Verify that the getter holds a token from the mind

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