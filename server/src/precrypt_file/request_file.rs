use std::ffi::OsStr;
use actix_web::client::Client;
use precrypt::{decrypt, DecryptionKeys};
use serde::{Deserialize, Serialize};

use umbral_pre::*;
use crate::precrypt_key::*;

#[derive(Serialize, Deserialize)]
pub struct FileRequest {
   file_cid: String,
   key_cid: String,
   sol_pubkey: Vec<u8>,         // sol pubkey
   sol_signed_message: Vec<u8>, // sol signed message
}

pub async fn request(
   req: FileRequest,
   request_uuid: String,
   orion_secret: String,
   _web3_token: String,
   threads: usize
) {
   // Files that will be used
   let raw_file_string = format!("{}/plaintext.bin", request_uuid); // TODO: Give this a proper file extension
   let raw_file_path = OsStr::new(&raw_file_string);
   let cipher_file_string = &format!("{}/cipher.bin", request_uuid);
   let cipher_file_path = OsStr::new(&cipher_file_string);

   let receiver_secret = SecretKey::random();
   // Get decryption keys
   let key_request = request_key::KeyRequest {
      cid: req.key_cid,
      precrypt_pubkey: receiver_secret.public_key().to_array().to_vec(),
      sol_pubkey: req.sol_pubkey, 
      sol_signed_message: req.sol_signed_message
   };
   let mut decryption_keys: DecryptionKeys = request_key::request(key_request, orion_secret).await.unwrap();

   // Get file from IFPS
   // TODO: Make this work for large files
   let client = Client::default();
   let file_response = client
      .get(format!("https://ipfs.io/ipfs/{}", req.file_cid))
      .timeout(std::time::Duration::new(20, 0))
      .send()
      .await;
   println!("{:?}", file_response);
   let file_response_bytes = file_response.unwrap().body().await.unwrap();
   let response_body_str: Vec<u8> = serde_json::from_slice(&file_response_bytes).unwrap();
   std::fs::write(cipher_file_path, response_body_str).unwrap();

   // Decrypt file with key
   decrypt(cipher_file_path, raw_file_path, receiver_secret, &mut decryption_keys, threads).unwrap();

   // Write decrypted file to disk
}
