import { Program, Provider, web3 } from '@project-serum/anchor';
import { Transaction, Keypair, clusterApiUrl, Cluster } from '@solana/web3.js';
import { NextApiRequest, NextApiResponse } from 'next';
import { Web3Storage, File } from 'web3.storage';
import initMiddleware from '../../lib/initMiddleware';
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
      case 'POST':
        // Get mint id from url param
        const mint = req.query['mint'];
        // TODO: Verify that poster owns mint


        const recrypt_key = req.body;
        // TODO: Validate that this is a properly formatted key

        
        
        let data = {
          "mint": mint,
          "recrypt_key": recrypt_key
        }
        
        // TODO: Encrypt this data before storing on IFPS

        // Store on IFPS with web3.storage
        const metadataBlob = Buffer.from(JSON.stringify(data));
        const web3_file = new File([metadataBlob], "data");
        const web3Client = new Web3Storage({ token: process.env.WEB_3_STORAGE_TOKEN });
        const cid = await web3Client.put([web3_file], { wrapWithDirectory: false })
        return res.end(JSON.stringify({"cid": cid}));
      default:
        res.setHeader('Allow', ['POST'])
        return res.status(405).end(`Method ${req.method} Not Allowed`)
    }
  } catch (err) {
    console.error(err);
    return res.status(500).send('Something went wrong');
  }
}

export const config = {
  api: {
    // bodyParser: false
      bodyParser: {
          sizeLimit: '10mb' // Set desired value here

      }
  }
}
