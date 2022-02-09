use precrypt::{precrypt, RecryptionKeys};
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use umbral_pre::*;

use crate::precrypt_key::*;

pub async fn store(
   file_uuid: String,
   mint: String,
   orion_secret: String,
   web3_token: String,
   threads: usize,
   mem_size: usize,
) {
   let raw_file_string = format!("{}/plaintext.bin", file_uuid);
   let raw_file_path = OsStr::new(&raw_file_string);

   // Encrypt file using precrypt
   println!("Encrypting...");
   let recrypt_key_string = format!("{}/recrypt.json", file_uuid);
   let recrypt_key_path = OsStr::new(&recrypt_key_string);
   let cipher_file_string = &format!("{}/cipher.bin", file_uuid);
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
   // TODO

   // Store Key
   let recryption_keys_array = std::fs::read(recrypt_key_string).unwrap();
   let recryption_keys: RecryptionKeys = serde_json::from_slice(&recryption_keys_array).unwrap();
   let key_store = store_key::KeyStoreRequest {
      recryption_keys: recryption_keys,
      mint: mint,
   };
   let response = store_key::store(key_store, orion_secret, web3_token)
      .await
      .unwrap();
   
   // Cleanup created files
   fs::remove_dir_all(&file_uuid).unwrap();

   // Write file CID and key CID to json in the folder with an expiration time
   std::fs::write("result.json", serde_json::to_string(&response).unwrap()).unwrap();
}
