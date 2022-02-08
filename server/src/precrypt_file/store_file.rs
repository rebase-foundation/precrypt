use precrypt::{ precrypt };
use std::fs;
use std::io::Write;
use std::ffi::OsStr;
use umbral_pre::*;

use crate::precrypt_key::*;

pub fn store(file_uuid: String, threads: usize, mem_size: usize) {
   let raw_file_string = format!("{}/plaintext.bin", file_uuid);
   let raw_file_path = OsStr::new(&raw_file_string);

   // Encrypt file using precrypt
   println!("Encrypting...");
   let recrypt_key_string = format!("{}/recrypt.json", file_uuid);
   let recrypt_key_path = OsStr::new(&recrypt_key_string);
   let cipher_file_string = &format!("{}/cipher.bin", file_uuid);
   let cipher_file_path = OsStr::new(&cipher_file_string);
   let file_key = SecretKey::random();
   precrypt(raw_file_path, file_key, &recrypt_key_path, cipher_file_path, threads, mem_size).unwrap();

   // Upload encrypted file to IPFS
   // TODO

   // Store Key
   // TODO
   // store_key::store(key_store_request: KeyStoreRequest, orion_secret: String, web3_token: String)
   // Cleanup created files
   fs::remove_dir_all(&file_uuid).unwrap();


   // Return CID?
}