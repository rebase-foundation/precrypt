use actix_web::client::Client;
use precrypt::decrypt;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::Path;
use std::process::Command;

use crate::precrypt_key::*;
use umbral_pre::*;

#[derive(Serialize, Deserialize)]
pub struct FileRequest {
   key_cid: String,
   sol_pubkey: Vec<u8>,         // sol pubkey
   sol_signed_message: Vec<u8>, // sol signed message
}

pub async fn request(
   req: FileRequest,
   request_uuid: String,
   orion_secret: String,
   web3_token: String,
   threads: usize,
) {
   let receiver_secret = SecretKey::random();
   // Get decryption keys
   let key_request = request_key::KeyRequest {
      key_cid: req.key_cid,
      precrypt_pubkey: receiver_secret.public_key().to_array().to_vec(),
      sol_pubkey: req.sol_pubkey,
      sol_signed_message: req.sol_signed_message,
   };
   let mut key_response: request_key::KeyResponse = request_key::request(key_request, orion_secret)
      .await
      .unwrap();

   // Get file from IFPS
   // TODO: Make this work for large files
   let client = Client::default();
   let file_response = client
      .get(format!("https://api.web3.storage/car/{}", key_response.file_cid))
      .header("authorization", format!("Bearer {}", web3_token))
      .timeout(std::time::Duration::new(120, 0))
      .send()
      .await;
   println!("{:?}", file_response);
   let file_response_bytes = file_response.unwrap().body().limit(10000000000).await.unwrap(); // without limit() this errors at > 256KB
   let cipher_car_string = &format!("{}/cipher.car", request_uuid);
   let cipher_car_path = OsStr::new(&cipher_car_string);
   std::fs::write(cipher_car_path, file_response_bytes).unwrap();
   
   println!("Unpacking cipher");
   let cipher_file_str = &format!("{}/cipher.txt", request_uuid);
   let cipher_file_path = OsStr::new(&cipher_file_str);
   let pack_command = format!(
      "npx ipfs-car --unpack {} --output {}",
      cipher_car_string, cipher_file_str
   );
   Command::new("sh")
      .arg("-c")
      .arg(pack_command)
      .output()
      .expect("failed to execute process");

   // Decrypt file with key
   // Write file CID and key CID to json in the folder with an expiration time
   if !Path::new("request_results").is_dir() {
      fs::create_dir("request_results").unwrap();
   }
   let raw_file_string = format!("request_results/{}.txt", request_uuid); // TODO: Give this a proper file extension
   let raw_file_path = OsStr::new(&raw_file_string);
   decrypt(
      &cipher_file_path,
      raw_file_path,
      receiver_secret,
      &mut key_response.decryption_keys,
      threads,
   )
   .unwrap();
   fs::remove_dir_all(&request_uuid).unwrap();
   println!("DONE!");
}
