use actix_web::client::Client;
use precrypt::{precrypt, RecryptionKeys};
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use umbral_pre::*;

use crate::precrypt_key::*;

pub async fn store(
   request_uuid: String,
   mint: String,
   orion_secret: String,
   web3_token: String,
   threads: usize,
   mem_size: usize,
) {
   let raw_file_string = format!("{}/plaintext.bin", request_uuid);
   let raw_file_path = OsStr::new(&raw_file_string);

   // Encrypt file using precrypt
   println!("Encrypting...");
   let recrypt_key_string = format!("{}/recrypt.json", request_uuid);
   let recrypt_key_path = OsStr::new(&recrypt_key_string);
   let cipher_file_string = &format!("{}/cipher.bin", request_uuid);
   let cipher_file_path = OsStr::new(&cipher_file_string);
   let file_key = SecretKey::random();
   precrypt(
      raw_file_path,
      file_key,
      &recrypt_key_path,
      cipher_file_path,
      threads,
      mem_size,
   )
   .unwrap();

   // Upload encrypted file to IPFS
   // TODO: Make this work for files > 100MB
   println!("Storing cipher...");
   let cipher_bytes = fs::read(cipher_file_path).unwrap();
   let cipher_str = serde_json::to_string(&cipher_bytes).unwrap();
   let client = Client::default();
   let file_response = client
      .post("https://api.web3.storage/upload")
      .header("authorization", format!("Bearer {}", web3_token))
      .timeout(std::time::Duration::new(20, 0))
      .send_body(&cipher_str)
      .await;
   println!("{:?}", file_response);
   let file_response_str = file_response.unwrap().body().await.unwrap();
   let file_response_json: Value = serde_json::from_slice(&file_response_str).unwrap();

   // Store Key
   println!("Storing key...");
   let recryption_keys_array = std::fs::read(recrypt_key_string).unwrap();
   let recryption_keys: RecryptionKeys = serde_json::from_slice(&recryption_keys_array).unwrap();
   let key_store = store_key::KeyStoreRequest {
      recryption_keys: recryption_keys,
      mint: mint,
   };
   let key_response_json = store_key::store(key_store, orion_secret, web3_token)
      .await
      .unwrap();
   
   // Cleanup created files
   fs::remove_dir_all(&request_uuid).unwrap();

   // Write file CID and key CID to json in the folder with an expiration time
   if !Path::new("store_requests").is_dir() {
      fs::create_dir("store_requests").unwrap();
   }
   let result_file_str = &format!("store_requests/{},json", request_uuid);
   std::fs::write(
      result_file_str,
      serde_json::to_string(&json!({
         "file_response": file_response_json,
         "key_response": key_response_json
      }))
      .unwrap(),
   )
   .unwrap();
}
