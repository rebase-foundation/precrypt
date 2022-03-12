use actix_web::client::Client;
use generic_array::GenericArray;
use nacl::sign::verify;
use orion::aead;
use precrypt::DecryptionKeys;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::Value;
use std::str;
use umbral_pre::*;

use crate::store_key::*;

#[derive(Serialize, Deserialize)]
pub struct KeyRequest {
    pub key_cid: String,
    pub precrypt_pubkey: Vec<u8>,    // recrypt key
    pub sol_pubkey: Vec<u8>,         // sol pubkey
    pub sol_signed_message: Vec<u8>, // sol signed message
}

#[derive(Serialize, Deserialize, Debug)]
struct SolanaJSONRPCResult {
    result: SolanaJSONRPCResultValue,
}

#[derive(Serialize, Deserialize, Debug)]
struct SolanaJSONRPCResultValue {
    value: Vec<Value>,
}

#[derive(Serialize, Deserialize)]
pub struct KeyResponse {
    pub file_cid: String,
    pub file_extension: String,
    pub decryption_keys: DecryptionKeys,
}

pub async fn request(request: KeyRequest, orion_secret: String) -> std::io::Result<KeyResponse> {
    // Get the data from IFPS
    let client = Client::default();
    let url = format!("https://{}.ipfs.dweb.link/", request.key_cid);
    println!("Getting url: {:?}", url);
    let response = client
        .get(url)
        .timeout(std::time::Duration::new(120, 0))
        .send()
        .await;
    println!("{:?}", response);
    let response_body_bytes = response.unwrap().body().await.unwrap();
    let response_body_str: String = serde_json::from_slice(&response_body_bytes).unwrap();
    let response_body: Vec<u8> = serde_json::from_str(&response_body_str).unwrap();

    // Decrypt the data with private key
    let secret_slice: Vec<u8> = serde_json::from_str(&orion_secret).unwrap();
    let secret_key = aead::SecretKey::from_slice(&secret_slice).unwrap();
    let decrypted_bytes = aead::open(&secret_key, &response_body).unwrap();
    let decrypted_str = str::from_utf8(&decrypted_bytes).unwrap();
    let data: KeyStoreRequest = serde_json::from_str(&decrypted_str).unwrap();
    let mint = data.mint;
    let file_cid = data.file_cid;
    let file_extension = data.file_extension;
    let recryption_keys = data.recryption_keys;

    println!("Mint {:?}", mint);

    // Verify that the getter holds the token
    // Verify signature
    let signed = verify(
        &request.sol_signed_message,
        "precrypt".as_bytes(),
        &request.sol_pubkey,
    )
    .unwrap();
    if !signed {
        panic!("Signature verification failed");
    }

    // Encode pubkey bytes to string
    let sol_pubkey = bs58::encode(request.sol_pubkey).into_string();

    // Verify solana pubkey owns token from mint
    let client = Client::default();
    let response = client
        .post("https://api.testnet.solana.com/")
        .header("Content-Type", "application/json")
        .send_body(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTokenAccountsByOwner",
            "params": [
                sol_pubkey,
                {
                    "mint": mint
                },
                {
                    "encoding": "jsonParsed"
                }
            ]
        }))
        .await;

    let response_body_bytes = response.unwrap().body().await.unwrap();
    let response_body: SolanaJSONRPCResult = serde_json::from_slice(&response_body_bytes).unwrap();
    println!("response_body, {:?}", response_body);
    let values = response_body.result.value;
    let mut owns_token = false;
    for value in values {
        println!("value, {:?}", value);
        let balance_str = value
            .get("account")
            .unwrap()
            .get("data")
            .unwrap()
            .get("parsed")
            .unwrap()
            .get("info")
            .unwrap()
            .get("tokenAmount")
            .unwrap()
            .get("amount")
            .unwrap();
        let balance: u64 = balance_str.as_str().unwrap().parse::<u64>().unwrap();
        if balance > 0 {
            owns_token = true;
        }
    }
    if !owns_token {
        panic!("Solana account doesn't own required token");
    }

    // Generate the decryption keys
    let precrypt_pubkey =
        PublicKey::from_array(&GenericArray::from_iter(request.precrypt_pubkey)).unwrap();
    let decryption_keys = precrypt::recrypt(recryption_keys, precrypt_pubkey).unwrap();
    let key_response = KeyResponse {
        file_cid: file_cid,
        file_extension: file_extension,
        decryption_keys: decryption_keys,
    };
    return Ok(key_response);
}
