use std::io::Write;
use crate::fs::OpenOptions;
use actix_web::client::Client;
use futures_util::StreamExt;
use precrypt::decrypt_file;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::precrypt_key::*;
use umbral_pre::*;

use crate::util::path_builder::{build_path, PathBuilder};
use crate::util::command::{run_command};

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
   // Read this as a stream to make it work for large files
   let client = Client::default();
   let mut response_stream = client
      .get(format!(
         "https://api.web3.storage/car/{}",
         key_response.file_cid
      ))
      .header("authorization", format!("Bearer {}", web3_token))
      .timeout(std::time::Duration::new(120, 0)) // TODO: This is probably an issue but it seems like its between chunks?
      .send()
      .await
      .unwrap();

   // Prep the file handle we will write to
   let cipher_car_path = build_path(PathBuilder::CipherCar, &request_uuid);
   let mut out = OpenOptions::new()
      .write(true)
      .append(true)
      .create_new(true)
      .open(&cipher_car_path)
      .unwrap();
   
   // Poll each chunk of the stream and append it to the handle
   while let Some(chunk) = response_stream.next().await {
      let chunk = chunk.unwrap();
      out.write(&chunk).unwrap();
   }
   // TODO: Error if this doesn't write the whole file

   println!("Unpacking cipher");
   let cipher_file_path = build_path(PathBuilder::Cipher, &request_uuid);
   run_command(format!(
      "ipfs-car --unpack {} --output {}",
      &cipher_car_path, cipher_file_path
   )).unwrap();

   // Decrypt file with key
   // Write file CID and key CID to json in the folder with an expiration time
   let results_dir = build_path(PathBuilder::RequestResultDir, &request_uuid);
   if !Path::new(&results_dir).is_dir() {
      fs::create_dir(&results_dir).unwrap();
   }
   let plaintext_path = build_path(PathBuilder::RequestResult, &request_uuid);
   // Add proper file name extension to path for later
   let plaintext_path = plaintext_path.replace(".bin", &format!(".{}.{}", &key_response.file_name, &key_response.file_extension));
   decrypt_file(
      &cipher_file_path,
      &plaintext_path,
      receiver_secret,
      &mut key_response.decryption_keys,
      threads,
   );
   fs::remove_dir_all(build_path(PathBuilder::TaskDir, &request_uuid)).unwrap();
   println!("DONE!");
}
