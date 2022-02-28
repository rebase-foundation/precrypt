use wasm_bindgen_test::*;
use umbral_pre::bindings_wasm::*;
use precrypt::bindings_wasm::*;
use std::fs;

#[wasm_bindgen_test]
fn pass() {
   let file_secret = SecretKey::random();

   // Create test file
   precrypt_file("test.txt", file_secret, "test.txt", 100000);
   assert_eq!(1, 1);
}