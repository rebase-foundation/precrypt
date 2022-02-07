use dotenv::dotenv;
use generic_array::GenericArray;
use std::env;
use umbral_pre::*;

use actix_cors::Cors;
use actix_web::client::Client;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use orion::aead;
use precrypt::RecryptionKeys;
use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use std::str;

// TODO: CORS?

#[derive(Serialize, Deserialize)]
struct UploadRequest {
    recryption_keys: RecryptionKeys,
    mint: String,
}

#[post("/upload")]
async fn upload(req_body: String) -> impl Responder {
    let request: UploadRequest = serde_json::from_str(&req_body).unwrap();

    // TODO: Verify that poster owns listing
    println!("{}", request.mint);

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
    requester_pubkey: Vec<u8>, // recrypt key
                               // sol pubkey
                               // sol signed message
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

    // TODO: Verify that the getter holds the token
    // Signed message
    // Public key
    // Verify (signed message, public key)
    println!("{}", mint);

    // Generate the decryption keys
    let requester_pubkey =
        PublicKey::from_array(&GenericArray::from_iter(request.requester_pubkey)).unwrap();
    let decryption_keys = precrypt::recrypt(recryption_keys, requester_pubkey).unwrap();
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
