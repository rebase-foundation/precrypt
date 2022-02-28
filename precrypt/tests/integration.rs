use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufReader;
use std::process::Command;

#[derive(Serialize, Deserialize, Clone)]
struct Keypair {
    public_key: Vec<u8>,
    secret_key: Vec<u8>,
}

#[test]
fn test_integration() {
   // Setup
   // Create seller key
   let output = Command::new(".././target/debug/precrypt")
      .args(["keygen", "tests/seller.json"])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
   // Create test file
   let test_data = "The crow flies at midnight.";
   fs::write("tests/secret.txt", test_data).unwrap();

   // Precrypt
   // Run encryption command
   let output = Command::new(".././target/debug/precrypt")
      .args([
         "encrypt",
         "tests/secret.txt",
         "tests/seller.json",
         "tests/recrypt.json",
         "tests/encrypted.txt",
         "-t",
         "1",
      ])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
   // Delete secret
   fs::remove_file("tests/secret.txt").unwrap();

   // Recrypt
   // Create buyer key
   let output = Command::new(".././target/debug/precrypt")
      .args(["keygen", "tests/buyer.json"])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
   let file = fs::File::open("tests/buyer.json").unwrap();
   let buyer_json: Keypair = serde_json::from_reader(BufReader::new(file)).unwrap();
   let buyer_pubkey_str = format!("{:?}", buyer_json.public_key);
   // Run recryption command
   let output = Command::new(".././target/debug/precrypt")
      .args([
         "recrypt",
         "tests/recrypt.json",
         &buyer_pubkey_str,
         "tests/decrypt.json",
      ])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );

   // Decrypt
   let output = Command::new(".././target/debug/precrypt")
      .args([
         "decrypt",
         "tests/encrypted.txt",
         "tests/decrypt.json",
         "tests/buyer.json",
         "tests/decrypted.txt",
         "-t",
         "1"
      ])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
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
fn test_integration_threaded() {
   // Setup
   // Create seller key
   let output = Command::new(".././target/debug/precrypt")
      .args(["keygen", "tests/t_seller.json"])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
   // Create test file
   let test_data = "The crow flies at midnight.";
   fs::write("tests/t_secret.txt", test_data).unwrap();

   // Precrypt
   // Run precryption command
   let output = Command::new(".././target/debug/precrypt")
      .args([
         "encrypt",
         "tests/t_secret.txt",
         "tests/t_seller.json",
         "tests/t_recrypt.json",
         "tests/t_encrypted.txt",
         "-m",
         "10",
         "-t",
         "2",
      ])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
   // Delete secret
   fs::remove_file("tests/t_secret.txt").unwrap();

   // Recrypt
   // Create buyer key
   let output = Command::new(".././target/debug/precrypt")
      .args(["keygen", "tests/t_buyer.json"])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
   let file = fs::File::open("tests/t_buyer.json").unwrap();
   let buyer_json: Keypair = serde_json::from_reader(BufReader::new(file)).unwrap();
   let buyer_pubkey_str = format!("{:?}", buyer_json.public_key);
   // Run recryption command
   let output = Command::new(".././target/debug/precrypt")
      .args([
         "recrypt",
         "tests/t_recrypt.json",
         &buyer_pubkey_str,
         "tests/t_decrypt.json",
      ])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );

   // Decrypt
   let output = Command::new(".././target/debug/precrypt")
      .args([
         "decrypt",
         "tests/t_encrypted.txt",
         "tests/t_decrypt.json",
         "tests/t_buyer.json",
         "tests/t_decrypted.txt",
      ])
      .output()
      .unwrap();
   assert_eq!(
      0,
      output.status.code().unwrap(),
      "{}",
      String::from_utf8_lossy(&output.stderr)
   );
   // Get contents of decrypted file
   let decrypted_data = fs::read_to_string("tests/t_decrypted.txt").unwrap();
   assert_eq!(test_data, decrypted_data);

   // Cleanup
   fs::remove_file("tests/t_seller.json").unwrap();
   fs::remove_file("tests/t_recrypt.json").unwrap();
   fs::remove_file("tests/t_encrypted.txt").unwrap();
   fs::remove_file("tests/t_buyer.json").unwrap();
   fs::remove_file("tests/t_decrypt.json").unwrap();
   fs::remove_file("tests/t_decrypted.txt").unwrap();
}
