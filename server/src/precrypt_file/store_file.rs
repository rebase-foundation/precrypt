use precrypt::{precrypt_file};
use serde_json::json;
use std::fs;
use std::path::Path;
use umbral_pre::*;

use crate::precrypt_key::*;

use crate::util::car::upload_cars;
use crate::util::command::run_command;
use crate::util::path_builder::{build_path, PathBuilder};

pub async fn store(
   request_uuid: String,
   network: String,
   mint: String,
   file_name: String,
   file_extension: String,
   orion_secret: String,
   web3_token: String,
   threads: usize,
   mem_size: usize,
) {
   let plaintext_path = build_path(PathBuilder::Plaintext, &request_uuid);

   // Encrypt file using precrypt
   println!("Encrypting...");
   let cipher_file_path = build_path(PathBuilder::Cipher, &request_uuid);
   let file_key = SecretKey::random();

   let recryption_keys = precrypt_file(
      &plaintext_path,
      file_key,
      &cipher_file_path,
      threads,
      mem_size,
   );

   // Prep encrypted file for IPFS
   println!("Prepping cipher");
   let cipher_car_path = build_path(PathBuilder::CipherCar, &request_uuid);
   run_command(format!(
      "ipfs-car --wrapWithDirectory false --pack {} --output {}",
      cipher_file_path, cipher_car_path
   ))
   .unwrap();
   run_command(format!(
      "carbites split --size 90MB --strategy treewalk {}",
      cipher_car_path
   ))
   .unwrap();

   // Upload encrypted file to IPFS
   println!("Storing cipher...");
   let file_root_cid = upload_cars(&request_uuid).await;

   // Store Key
   println!("Storing key...");
   let file_cid: String = file_root_cid.to_string();
   let key_store = store_key::KeyStoreRequest {
      recryption_keys: recryption_keys,
      network: network,
      mint: mint,
      file_cid: file_cid,
      file_name: file_name,
      file_extension: file_extension,
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
