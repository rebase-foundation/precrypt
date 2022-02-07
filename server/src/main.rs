use dotenv::dotenv;
use generic_array::GenericArray;
use std::env;
use umbral_pre::*;

use actix_cors::Cors;
use actix_web::client::Client;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use nacl::sign::verify;
use orion::aead;
use precrypt::RecryptionKeys;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::value::Value;
use std::str;

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    recryption_keys: RecryptionKeys,
    mint: String,
}

#[post("/upload")]
async fn upload(req_body: String) -> impl Responder {
    let _request: UploadRequest = serde_json::from_str(&req_body).unwrap();

    // Encrypt the data before storing on IPFS
    let secret_string = env::var("ORION_SECRET").unwrap();
    let secret_slice: Vec<u8> = serde_json::from_str(&secret_string).unwrap();
    let secret_key = aead::SecretKey::from_slice(&secret_slice).unwrap();
    let cipher_bytes = aead::seal(&secret_key, req_body.as_bytes()).unwrap();
    let cipher_str = serde_json::to_string(&cipher_bytes).unwrap();

    // Store the data on IPFS with web3.storage
    let client = Client::default();
    let token = env::var("WEB3").unwrap();
    let response = client
        .post("https://api.web3.storage/upload")
        .header("authorization", format!("Bearer {}", token))
        .send_json(&cipher_str)
        .await;

    let response_body_str = response.unwrap().body().await.unwrap();
    let response_body: Value = serde_json::from_slice(&response_body_str).unwrap();
    return HttpResponse::Ok().body(response_body);
}

#[derive(Serialize, Deserialize)]
struct RecryptRequest {
    cid: String,
    precrypt_pubkey: Vec<u8>,    // recrypt key
    sol_pubkey: Vec<u8>,         // sol pubkey
    sol_signed_message: Vec<u8>, // sol signed message
}

#[derive(Serialize, Deserialize)]
struct SolanaJSONRPCResult {
    result: SolanaJSONRPCResultValue
}

#[derive(Serialize, Deserialize)]
struct SolanaJSONRPCResultValue {
    value: Vec<Value> 
}

#[post("/download")]
async fn download(req_body: String) -> impl Responder {
    let request: RecryptRequest = serde_json::from_str(&req_body).unwrap();

    // Get the data from IFPS
    let client = Client::default();
    let response = client
        .get(format!("https://ipfs.io/ipfs/{}", request.cid))
        .send()
        .await;

    let response_body_bytes = response.unwrap().body().await.unwrap();
    let response_body_str: String = serde_json::from_slice(&response_body_bytes).unwrap();
    let response_body: Vec<u8> = serde_json::from_str(&response_body_str).unwrap();
    // Decrypt the data with private key
    let secret_string = env::var("ORION_SECRET").unwrap();
    let secret_slice: Vec<u8> = serde_json::from_str(&secret_string).unwrap();
    let secret_key = aead::SecretKey::from_slice(&secret_slice).unwrap();
    let decrypted_bytes = aead::open(&secret_key, &response_body).unwrap();
    let decrypted_str = str::from_utf8(&decrypted_bytes).unwrap();
    let data: UploadRequest = serde_json::from_str(&decrypted_str).unwrap();
    let mint = data.mint;
    let recryption_keys = data.recryption_keys;

    // Verify that the getter holds the token
    // Verify signature
    let signed = verify(
        &request.sol_signed_message,
        "precrypt".as_bytes(),
        &request.sol_pubkey,
    )
    .unwrap();
    if !signed {
        return HttpResponse::Unauthorized().body("Signature verification failed");
    }

    // Encode pubkey bytes to string
    let sol_pubkey = bs58::encode(request.sol_pubkey).into_string();

    // Verify solana pubkey owns token from mint
    let client = Client::default();
    let response = client
        .post("https://ssc-dao.genesysgo.net/")
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
    let values = response_body.result.value;
    let mut owns_token = false;
    for value in values {
        let balance_str = value
            .get("account").unwrap()
            .get("data").unwrap()
            .get("parsed").unwrap()
            .get("info").unwrap()
            .get("tokenAmount").unwrap()
            .get("uiAmountString").unwrap();
        let balance: f64 = balance_str.as_str().unwrap().parse::<f64>().unwrap();
        if balance >= 1.0 {
            owns_token = true;
        }
    }
    if !owns_token {
        return HttpResponse::Unauthorized().body("Solana account doesn't own required token");
    }

    // Generate the decryption keys
    let precrypt_pubkey =
        PublicKey::from_array(&GenericArray::from_iter(request.precrypt_pubkey)).unwrap();
    let decryption_keys = precrypt::recrypt(recryption_keys, precrypt_pubkey).unwrap();
    return HttpResponse::Ok().body(serde_json::to_string(&decryption_keys).unwrap());
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let host = match env::var("SERVER_HOST") {
        Ok(host) => host,
        Err(_e) => "0.0.0.0:8000".to_string(),
    };

    println!("Starting server on {:?}", host);
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_header()
            .allowed_methods(vec!["POST"])
            .max_age(3600);

        App::new().wrap(cors).service(upload).service(download)
    })
    .bind(host)?
    .run()
    .await
}