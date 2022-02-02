use serde_json::Value;
use std::io::BufReader;
use std::fs;
use std::process::Command;

#[test]
fn test_integration() {
   // Setup
   // Create seller key
   let output = Command::new("./target/debug/precrypt").args(["keygen", "tests/seller.json"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   // Create test file
   let test_data = "The crow flies at midnight.";
   fs::write("tests/secret.txt", test_data).unwrap();

   // Precrypt
   // Run precryption command
   let output = Command::new("./target/debug/precrypt").args(["precrypt", "tests/secret.txt", "tests/seller.json", "tests/recrypt.json", "tests/encrypted.txt"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   // Delete secret
   fs::remove_file("tests/secret.txt").unwrap();

   // Recrypt
   // Create buyer key
   let output = Command::new("./target/debug/precrypt").args(["keygen", "tests/buyer.json"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   let file = fs::File::open("tests/buyer.json").unwrap();
   let buyer_json: Value = serde_json::from_reader(BufReader::new(file)).unwrap();
   let buyer_pubkey = buyer_json["public_key"].as_str().unwrap();
   // Run recryption command
   let output = Command::new("./target/debug/precrypt").args(["recrypt", "tests/recrypt.json", buyer_pubkey, "tests/decrypt.json"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));

   // Decrypt
   let output = Command::new("./target/debug/precrypt").args(["decrypt", "tests/encrypted.txt", "tests/decrypt.json", "tests/buyer.json", "tests/decrypted.txt"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   // Get contents of decrypted file
   let decrypted_data = fs::read_to_string("tests/decrypted.txt").unwrap();
   assert_eq!(test_data, decrypted_data);

   // Cleanup
   fs::remove_file("tests/seller.json").unwrap();
   fs::remove_file("tests/recrypt.json").unwrap();
   fs::remove_file("tests/encrypted.txt").unwrap();
   fs::remove_file("tests/buyer.json").unwrap();
   fs::remove_file("tests/decrypt.json").unwrap();
   fs::remove_file("tests/decrypted.txt").unwrap();
}

#[test]
fn test_integration_large_file() {
   // Setup
   // Create seller key
   let output = Command::new("./target/debug/precrypt").args(["keygen", "tests/seller.json"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   // Create test file
   let test_data = "The crow flies at midnight.".repeat(1000);
   fs::write("tests/secret.txt", test_data).unwrap();

   // Precrypt
   // Run precryption command
   let output = Command::new("./target/debug/precrypt").args(["precrypt", "tests/secret.txt", "tests/seller.json", "tests/recrypt.json", "tests/encrypted.txt"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   // Delete secret
   fs::remove_file("tests/secret.txt").unwrap();

   // Recrypt
   // Create buyer key
   let output = Command::new("./target/debug/precrypt").args(["keygen", "tests/buyer.json"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   let file = fs::File::open("tests/buyer.json").unwrap();
   let buyer_json: Value = serde_json::from_reader(BufReader::new(file)).unwrap();
   let buyer_pubkey = buyer_json["public_key"].as_str().unwrap();
   // Run recryption command
   let output = Command::new("./target/debug/precrypt").args(["recrypt", "tests/recrypt.json", buyer_pubkey, "tests/decrypt.json"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));

   // Decrypt
   let output = Command::new("./target/debug/precrypt").args(["decrypt", "tests/encrypted.txt", "tests/decrypt.json", "tests/buyer.json", "tests/decrypted.txt"]).output().unwrap();
   assert_eq!(0, output.status.code().unwrap(), "Error: {}", String::from_utf8_lossy(&output.stderr));
   // Get contents of decrypted file
   let decrypted_data = fs::read_to_string("tests/decrypted.txt").unwrap();
   assert_eq!(test_data, decrypted_data);

   // Cleanup
   fs::remove_file("tests/seller.json").unwrap();
   fs::remove_file("tests/recrypt.json").unwrap();
   fs::remove_file("tests/encrypted.txt").unwrap();
   fs::remove_file("tests/buyer.json").unwrap();
   fs::remove_file("tests/decrypt.json").unwrap();
   fs::remove_file("tests/decrypted.txt").unwrap();
}