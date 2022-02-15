use actix_web::client::Client;
use glob::glob;
use precrypt::{precrypt, RecryptionKeys};
use serde_json::{json, Value};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use umbral_pre::*;

use crate::precrypt_key::*;

use crate::util::path_builder::{build_path, PathBuilder};
use crate::util::command::{run_command};

pub async fn store(
   request_uuid: String,
   mint: String,
   file_extension: String,
   orion_secret: String,
   web3_token: String,
   threads: usize,
   mem_size: usize,
) {
   let plaintext_path = build_path(PathBuilder::Plaintext, &request_uuid);

   // Encrypt file using precrypt
   println!("Encrypting...");
   let recrypt_key_path = build_path(PathBuilder::RecryptKey, &request_uuid);
   let cipher_file_path = build_path(PathBuilder::Cipher, &request_uuid);
   let file_key = SecretKey::random();
   precrypt(
      OsStr::new(&plaintext_path),
      file_key,
      OsStr::new(&recrypt_key_path),
      OsStr::new(&cipher_file_path),
      threads,
      mem_size,
   )
   .unwrap();

   // Prep encrypted file for IPFS
   // TODO: CHECK STD_ERRS FROM COMMANDS
   println!("Prepping cipher");
   let cipher_car_path = build_path(PathBuilder::CipherCar, &request_uuid);
   run_command(format!(
      "npx ipfs-car --wrapWithDirectory false --pack {} --output {}",
      cipher_file_path, cipher_car_path
   )).unwrap();
   run_command(format!(
      "npx carbites-cli split --size 90MB --strategy treewalk {}",
      cipher_car_path
   )).unwrap();

   // Get root cid for the cars
   let output = run_command(format!("npx ipfs-car --list-roots {}", cipher_car_path)).unwrap();
   let file_root_cid = std::str::from_utf8(&output.stdout).unwrap().replace("\n", "");
   println!("Root CID: {}", file_root_cid);

   // Upload encrypted file to IPFS
   println!("Storing cipher...");
   let pattern = build_path(PathBuilder::CarPattern, &request_uuid);
   for entry in glob(&pattern).expect("Failed to read glob pattern") {
      let path = entry.unwrap();
      println!("{:?}", path);
      let cipher_bytes = fs::read(path).unwrap();
      let body = actix_web::web::Bytes::from_iter(cipher_bytes);
      // TODO: Resilient client with retries
      let client = Client::default();
      let file_response = client
         .post("https://api.web3.storage/car")
         .header("authorization", format!("Bearer {}", web3_token))
         .timeout(std::time::Duration::new(120, 0))
         .send_body(body)
         .await;
      println!("{:?}", file_response);
      let file_response_str = file_response.unwrap().body().await.unwrap();
      let json: Value = serde_json::from_slice(&file_response_str).unwrap();
      if file_root_cid.eq(&json["cid"].to_string()) {
         let msg = format!("Received CID different from root: {}", json["cid"].to_string());
         panic!("{}", msg);
      }
    }

   // Store Key
   println!("Storing key...");
   let file_cid: String = file_root_cid.to_string();
   let recryption_keys_array = std::fs::read(recrypt_key_path).unwrap();
   let recryption_keys: RecryptionKeys = serde_json::from_slice(&recryption_keys_array).unwrap();
   let key_store = store_key::KeyStoreRequest {
      recryption_keys: recryption_keys,
      mint: mint,
      file_cid: file_cid,
      file_extension: file_extension
   };
   let key_response_json = store_key::store(key_store, orion_secret, web3_token)
      .await
      .unwrap();
   let key_cid = key_response_json["cid"].to_string().replace("\"", "");
   
   // Cleanup created files
   fs::remove_dir_all(build_path(PathBuilder::TaskDir, &request_uuid)).unwrap();

   // Write file CID and key CID to json in the folder with an expiration time
   let results_dir = build_path(PathBuilder::StoreResultDir, &request_uuid);
   if !Path::new(&results_dir).is_dir() {
      fs::create_dir(&results_dir).unwrap();
   }
   let result_file_str = build_path(PathBuilder::StoreResult, &request_uuid);
   std::fs::write(
      result_file_str,
      serde_json::to_string(&json!({
         "file_cid": file_root_cid,
         "key_cid": key_cid
      }))
      .unwrap(),
   )
   .unwrap();
   println!("DONE");
}
