use umbral_pre::*;
use generic_array::GenericArray;

fn main() -> std::io::Result<()> {
   // READ INPUTS
   println!("Inputs:");
   let plaintext_bytes = std::fs::read("../plaintext.txt").unwrap();
   let plaintext_str = format!("{:?}", plaintext_bytes);
   println!("   Plaintext: {}", plaintext_str.replace(" ", ""));
   let secret_raw = std::fs::read("../secret.json").unwrap();
   let secret_bytes: Vec<u8> = serde_json::from_slice(&secret_raw).unwrap();
   let secret_str = format!("{:?}", secret_bytes);
   println!("   Secret: {}", secret_str.replace(" ", ""));
   let secret: SecretKey = SecretKey::from_array(&GenericArray::from_iter(secret_bytes)).unwrap();
   
   // ENCRYPT
   let (capsule, cipher_chunk) = encrypt(&secret.public_key(), &plaintext_bytes).unwrap();
   
   // PRINT OUTPUTS
   println!("\nOutputs:");
   let capsule_str = format!("{:?}", capsule.to_array().to_vec());
   println!("   Capsule: {}", capsule_str.replace(" ", ""));
   let cipher_str = format!("{:?}", cipher_chunk.to_vec());
   println!("   Cipher: {}", cipher_str.replace(" ", ""));

   // WRITE CAPSULE
   std::fs::write("../rust_capsule.json", capsule_str.replace(" ", ""))?;

   // TEST RUST CAPSULE
   println!("");
   let rust_capsule_raw = std::fs::read("../rust_capsule.json").unwrap();
   let rust_capsule_bytes: Vec<u8> = serde_json::from_slice(&rust_capsule_raw).unwrap();
   println!("Rust capsule length: {}", rust_capsule_bytes.len());
   match Capsule::from_array(&GenericArray::from_iter(rust_capsule_bytes)) {
      Ok(_) => {println!("Successfully parsed Rust capsule")},
      Err(_) => {println!("FAILED parsing Rust capsule")}
   }
   
   // TEST JS CAPSULE
   let js_capsule_raw = std::fs::read("../js_capsule.json").unwrap();
   let js_capsule_bytes: Vec<u8> = serde_json::from_slice(&js_capsule_raw).unwrap();
   println!("JS capsule length: {}", js_capsule_bytes.len());
   match Capsule::from_array(&GenericArray::from_iter(js_capsule_bytes)) {
      Ok(_) => {println!("Successfully parsed JS capsule")},
      Err(_) => {println!("FAILED parsing JS capsule")}
   }
   Ok(())
}