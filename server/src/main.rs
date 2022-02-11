use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use futures_util::stream::StreamExt as _;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use uuid::Uuid;

mod precrypt_key;
use crate::precrypt_key::*;

mod precrypt_file;
use crate::precrypt_file::*;

const THREADS: usize = 10;
const MEM_SIZE: usize = 50000000;

#[post("/file/store")]
async fn file_store(mut payload: Multipart) -> impl Responder {
    // UUID for request
    let request_uuid: String = format!("store-{}", Uuid::new_v4().to_simple().to_string());
    fs::create_dir(&request_uuid).unwrap();
    let raw_file_string = format!("{}/plaintext.bin", request_uuid);
    let raw_file_path = OsStr::new(&raw_file_string);

    // Write file to disk using multipart stream
    let mut file_count = 0;
    let mut mint: Option<String> = None;
    while let Some(item) = payload.next().await {
        file_count += 1;
        match file_count {
            1 => {
                let mut mint_field = item.unwrap();
                let field_content = mint_field.content_disposition().unwrap();
                let field_name = field_content.get_name().unwrap();
                if field_name != "mint" {
                    panic!("Invalid field: expected 'mint'")
                }

                let mut bytes: Vec<u8> = Vec::new();
                while let Some(chunk) = mint_field.next().await {
                    bytes.append(&mut chunk.unwrap().to_vec());
                }
                mint = Some(std::str::from_utf8(&bytes).unwrap().to_string());
            }
            2 => {
                let mut field = item.unwrap();
                println!(
                    "Uploading: {}",
                    field.content_disposition().unwrap().get_filename().unwrap()
                );
                let mut out = fs::OpenOptions::new()
                    .write(true)
                    .append(true)
                    .create_new(true)
                    .open(raw_file_path)
                    .unwrap();
                while let Some(chunk) = field.next().await {
                    out.write(&chunk.unwrap()).unwrap();
                }
            }
            _ => panic!("Invalid form data"),
        }
    }

    let request_uuid_c = request_uuid.clone();
    let orion_string = env::var("ORION_SECRET").unwrap();
    let web3_token = env::var("WEB3").unwrap();
    let mint = mint.unwrap().clone();
    actix_web::rt::spawn(async move {
        store_file::store(
            request_uuid_c,
            mint,
            orion_string,
            web3_token,
            THREADS,
            MEM_SIZE,
        )
        .await;
    });
    return HttpResponse::Ok().body(request_uuid);
}

#[post("/file/request")]
async fn file_request(req_body: String) -> impl Responder {
    let req: request_file::FileRequest = serde_json::from_str(&req_body).unwrap();
    // UUID for request
    let request_uuid: String = format!("request-{}", Uuid::new_v4().to_simple().to_string());
    fs::create_dir(&request_uuid).unwrap();

    // Spawn the worker for request
    let orion_string = env::var("ORION_SECRET").unwrap();
    let web3_token = env::var("WEB3").unwrap();
    let request_uuid_c = request_uuid.clone();
    actix_web::rt::spawn(async move {
        request_file::request(req, request_uuid_c, orion_string, web3_token, THREADS).await;
    });

    // Get the uuid
    return HttpResponse::Ok().body(request_uuid);
}

// TODO: Status endpoint that takes UUID and serves status/results then clears files
#[derive(Serialize, Deserialize)]
struct FileStatusBody {
    uuid: String,
}

#[get("file/status")]
async fn file_status(req_body: String) -> impl Responder {
    let body: FileStatusBody = serde_json::from_str(&req_body).unwrap();
    let (prefix, _) = body.uuid.split_once("-").unwrap();
    match prefix {
        "store" => {
            let status = status_file::store_status(body.uuid);
            match status {
                status_file::StoreStatus::EncryptingPlaintext => {
                    return HttpResponse::Ok().body("Encrypting plaintext with precrypt")
                }
                status_file::StoreStatus::ProcessingCipher => {
                    return HttpResponse::Ok().body("Process cipher for storage on IPFS")
                }
                status_file::StoreStatus::UploadingCipher => {
                    return HttpResponse::Ok().body("Uploading cipher to IPFS")
                }
                status_file::StoreStatus::Ready => return HttpResponse::Ok().body("Ready"),
                status_file::StoreStatus::NotFound => {
                    return HttpResponse::Ok().body("Task with uuid not found")
                }
            }
        }
        "request" => {
            println!("request")
        }
        _ => panic!("Invalid uuid"),
    }
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
    let request: store_key::KeyStoreRequest = serde_json::from_str(&req_body).unwrap();
    let orion_string = env::var("ORION_SECRET").unwrap();
    let web3_token = env::var("WEB3").unwrap();
    let response = store_key::store(request, orion_string, web3_token)
        .await
        .unwrap();
    return HttpResponse::Ok().body(response);
}

#[post("/key/request")]
async fn key_request(req_body: String) -> impl Responder {
    let request: request_key::KeyRequest = serde_json::from_str(&req_body).unwrap();
    let secret_string = env::var("ORION_SECRET").unwrap();
    let key_response: request_key::KeyResponse =
        request_key::request(request, secret_string).await.unwrap();
    let response = serde_json::to_string(&key_response).unwrap();
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
            .service(file_store)
            .service(file_request)
            .service(file_status)
    })
    .bind(host)?
    .run()
    .await
}
