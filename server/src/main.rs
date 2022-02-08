use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use futures_util::stream::StreamExt as _;
use precrypt::{ precrypt };
use std::env;
use std::fs;
use std::io::Write;
use std::ffi::OsStr;
use umbral_pre::*;
use uuid::Uuid;

mod precrypt_keys;
use crate::precrypt_keys::*;

const THREADS: usize = 10;
const MEM_SIZE: usize = 50000000;

#[post("/file/store")]
async fn store_file(mut payload: Multipart) -> impl Responder {
    let file_uuid: String = Uuid::new_v4().to_simple().to_string();
    fs::create_dir(&file_uuid).unwrap();

    let raw_file_string = format!("{}/plaintext.bin", file_uuid);
    let raw_file_path = OsStr::new(&raw_file_string);

    // Write file to disk using multipart stream
    let mut file_count = 0;
    while let Some(item) = payload.next().await {
        file_count += 1;
        if file_count > 1 {
            return HttpResponse::BadRequest().body("Only submit one file at a time.");
        }
        let mut field = item.unwrap();
        println!("Uploading: {}", field.content_disposition().unwrap().get_filename().unwrap());

        let mut out = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create_new(true)
            .open(raw_file_path)
            .unwrap();

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            // std::fs::write(path: P, contents: C)
            out.write(&chunk.unwrap()).unwrap();
            // println!("-- CHUNK: \n{:?}", std::str::from_utf8(&chunk.unwrap()));
        }
    }

    // Encrypt file using precrypt
    println!("Encrypting...");
    let recrypt_key_string = format!("{}/recrypt.json", file_uuid);
    let recrypt_key_path = OsStr::new(&recrypt_key_string);
    let cipher_file_string = &format!("{}/cipher.bin", file_uuid);
    let cipher_file_path = OsStr::new(&cipher_file_string);
    let file_key = SecretKey::random();
    precrypt(raw_file_path, file_key, &recrypt_key_path, cipher_file_path, THREADS, MEM_SIZE).unwrap();

    // Upload encrypted file to IPFS
    // TODO

    // Cleanup created files
    fs::remove_dir_all(&file_uuid).unwrap();

    // Store Keys
    

    // Return CID?

    return HttpResponse::Ok().body("OK");
}

// Generates Orion keys to be used for IPFS storage
#[get("/keygen")]
async fn keygen() -> impl Responder {
    let secret_key = orion::aead::SecretKey::default();
    let secret_key_str = serde_json::to_string(&secret_key.unprotected_as_bytes()).unwrap();
    return HttpResponse::Ok().body(&secret_key_str);
}

#[post("/key/store")]
async fn key_store(req_body: String) -> impl Responder {
    let request: store::KeyStoreRequest = serde_json::from_str(&req_body).unwrap();
    let orion_string = env::var("ORION_SECRET").unwrap();
    let web3_token = env::var("WEB3").unwrap();
    let response = precrypt_keys::store::store(request, orion_string, web3_token).await.unwrap();
    return HttpResponse::Ok().body(response);
}

#[post("/key/request")]
async fn key_request(req_body: String) -> impl Responder {
    let request: request::RecryptRequest = serde_json::from_str(&req_body).unwrap();
    let secret_string = env::var("ORION_SECRET").unwrap();
    let response = request::request(request, secret_string).await.unwrap();
    return HttpResponse::Ok().body(response);
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

        App::new()
            .wrap(cors)
            .service(key_store)
            .service(key_request)
            .service(keygen)
            .service(store_file)
    })
    .bind(host)?
    .run()
    .await
}
