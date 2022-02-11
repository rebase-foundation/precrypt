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
   file_extension: String,
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
   // TODO: CHECK STD_ERRS FROM COMMANDS
   println!("Prepping cipher");
   let cipher_car_string = format!("{}/cipher.car", request_uuid);
   let pack_command = format!(
      "npx ipfs-car --wrapWithDirectory false --pack {} --output {}",
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

   // Get root cid for the cars
   let get_cid_command = format!("npx ipfs-car --list-roots {}/cipher-0.car", request_uuid);
   let output = Command::new("sh")
      .arg("-c")
      .arg(get_cid_command)
      .output()
      .expect("failed to execute process");
   let file_root_cid = std::str::from_utf8(&output.stdout).unwrap().replace("\n", "");
   println!("Root CID: {}", file_root_cid);

   // Upload encrypted file to IPFS
   println!("Storing cipher...");
   let pattern = format!("./{}/cipher-*.car", request_uuid);
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
      if  file_root_cid.eq(&json["cid"].to_string()) {
         let msg = format!("Received CID different from root: {}", json["cid"].to_string());
         panic!("{}", msg);
      }
    }

   // Store Key
   println!("Storing key...");
   let file_cid: String = file_root_cid.to_string();
   let recryption_keys_array = std::fs::read(recrypt_key_string).unwrap();
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
   
   // Cleanup created files
   fs::remove_dir_all(&request_uuid).unwrap();

   // Write file CID and key CID to json in the folder with an expiration time
   if !Path::new("store_results").is_dir() {
      fs::create_dir("store_results").unwrap();
   }
   let result_file_str = &format!("store_results/{}.json", request_uuid);
   std::fs::write(
      result_file_str,
      serde_json::to_string(&json!({
         "file_response": file_root_cid,
         "key_response": key_response_json
      }))
      .unwrap(),
   )
   .unwrap();
   println!("DONE");
}
