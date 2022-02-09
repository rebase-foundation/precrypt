use actix_cors::Cors;
use actix_multipart::Multipart;
use futures_util::stream::StreamExt as _;
use actix_web::{get, post, App, HttpResponse, HttpServer, Responder};
use dotenv::dotenv;
use std::env;
use std::fs;
use std::io::Write;
use uuid::Uuid;
use std::ffi::OsStr;

mod precrypt_key;
use crate::precrypt_key::*;

mod precrypt_file;
use crate::precrypt_file::*;

const THREADS: usize = 10;
const MEM_SIZE: usize = 50000000;

#[post("/file/store")]
async fn file_store(mut payload: Multipart) -> impl Responder {
    let file_uuid: String = Uuid::new_v4().to_simple().to_string();
    fs::create_dir(&file_uuid).unwrap();

    let raw_file_string = format!("{}/plaintext.bin", file_uuid);
    let raw_file_path = OsStr::new(&raw_file_string);

    // Write file to disk using multipart stream
    let mut file_count = 0;
    let mut mint: Option<String> = None;
    while let Some(item) = payload.next().await {
        file_count += 1;
        match file_count {
            1 => {
                println!("mint");
                let mut mint_field = item.unwrap();
                let field_content = mint_field.content_disposition().unwrap();
                let field_name = field_content.get_name().unwrap();
                if field_name != "mint" { panic!("Invalid field: expected 'mint'")}

                let mut bytes: Vec<u8> = Vec::new();
                while let Some(chunk) = mint_field.next().await {
                    bytes.append(&mut chunk.unwrap().to_vec());
                }
                mint = Some(std::str::from_utf8(&bytes).unwrap().to_string());
            },
            2 => {
                println!("file");
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
            },
            _ => panic!("Invalid form data")
        }
    }

    let file_uuid_c = file_uuid.clone();
    let orion_string = env::var("ORION_SECRET").unwrap();
    let web3_token = env::var("WEB3").unwrap();
    let mint = mint.unwrap().clone();
    actix_web::rt::spawn(async move {
        store_file::store(file_uuid_c, mint, orion_string, web3_token, THREADS, MEM_SIZE).await;
    });
    return HttpResponse::Ok().body(file_uuid);
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
    let request: request_key::RecryptRequest = serde_json::from_str(&req_body).unwrap();
    let secret_string = env::var("ORION_SECRET").unwrap();
    let response = request_key::request(request, secret_string).await.unwrap();
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
    })
    .bind(host)?
    .run()
    .await
}
