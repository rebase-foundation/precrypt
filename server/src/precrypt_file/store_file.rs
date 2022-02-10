use actix_web::client::Client;
use glob::glob;
use precrypt::{precrypt, RecryptionKeys};
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;
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

   // Prep encrypted file for IPFS
   println!("Prepping cipher");
   let cipher_car_string = format!("{}/cipher.car", request_uuid);
   let pack_command = format!(
      "npx ipfs-car --pack {} --output {}",
      cipher_file_string, cipher_car_string
   );
   Command::new("sh")
      .arg("-c")
      .arg(pack_command)
      .output()
      .expect("failed to execute process");
   let carbites_command = format!(
      "npx carbites-cli split --size 10MB --strategy treewalk {}",
      cipher_car_string
   );
   Command::new("sh")
      .arg("-c")
      .arg(carbites_command)
      .output()
      .expect("failed to execute process");

   // Upload encrypted file to IPFS
   // TODO: Make this work for files > 100MB
   println!("Storing cipher...");
   let cipher_car_string = format!("{}/cipher-0.car", request_uuid);
   let cipher_bytes = fs::read(cipher_car_string).unwrap();
   let body = actix_web::web::Bytes::from_iter(cipher_bytes);
   let client = Client::default();
   let file_response = client
      .post("https://api.web3.storage/upload")
      .header("authorization", format!("Bearer {}", web3_token))
      .timeout(std::time::Duration::new(20, 0))
      .send_body(body)
      .await;
   println!("{:?}", file_response);
   let file_response_str = file_response.unwrap().body().await.unwrap();
   let file_response_json: Value = serde_json::from_slice(&file_response_str).unwrap();
   
//    let pattern = format!("./{}/cipher-*.car", request_uuid);
//    let mut file_response_json: Option<Value> = None;
//    for entry in glob(&pattern).expect("Failed to read glob pattern") {
//       let path = entry.unwrap();
//       println!("{:?}", path);
      
//       let car_bytes = fs::read(&path).unwrap();
//       let client = Client::default();
//       let body = actix_web::web::Bytes::from_iter(car_bytes);
//       let file_response = client
//          .post("https://api.web3.storage/car")
//          .header("authorization", format!("Bearer {}", web3_token))
//          .header("Content-Type", "application/car")
//          .timeout(std::time::Duration::new(120, 0))
//          .send_body(body)
//          .await;
//       println!("{:?}", file_response);
//       let file_response_str = file_response.unwrap().body().await.unwrap();
//       let json: Value = serde_json::from_slice(&file_response_str).unwrap();
//       file_response_json = Some(json);
//   }

   // Store Key
   println!("Storing key...");
   let file_cid: String = serde_json::from_str(&file_response_json["cid"].to_string()).unwrap();
   let recryption_keys_array = std::fs::read(recrypt_key_string).unwrap();
   let recryption_keys: RecryptionKeys = serde_json::from_slice(&recryption_keys_array).unwrap();
   let key_store = store_key::KeyStoreRequest {
      recryption_keys: recryption_keys,
      mint: mint,
      file_cid: file_cid
   };
   let key_response_json = store_key::store(key_store, orion_secret, web3_token)
      .await
      .unwrap();
   
   // Cleanup created files
   fs::remove_dir_all(&request_uuid).unwrap();

   // Write file CID and key CID to json in the folder with an expiration time
   if !Path::new("store_results").is_dir() {
      fs::create_dir("store_results").unwrap();
   }
   let result_file_str = &format!("store_results/{},json", request_uuid);
   std::fs::write(
      result_file_str,
      serde_json::to_string(&json!({
         "file_response": file_response_json,
         "key_response": key_response_json
      }))
      .unwrap(),
   )
   .unwrap();
   println!("DONE");
}
