use precrypt::RecryptionKeys;
use orion::aead;
use serde::{Deserialize, Serialize};
use actix_web::client::Client;
use serde_json::value::Value;

#[derive(Serialize, Deserialize)]
pub struct KeyStoreRequest {
    pub recryption_keys: RecryptionKeys,
    pub mint: String,
}

pub async fn store(key_store_request: KeyStoreRequest, orion_secret: String, web3_token: String) -> std::io::Result<Value> {
   // Encrypt the data before storing on IPFS
   let secret_slice: Vec<u8> = serde_json::from_str(&orion_secret).unwrap();
   let secret_key = aead::SecretKey::from_slice(&secret_slice).unwrap();
   let cipher_bytes = aead::seal(&secret_key, &serde_json::to_vec(&key_store_request).unwrap()).unwrap();
   let cipher_str = serde_json::to_string(&cipher_bytes).unwrap();

   // Store the data on IPFS with web3.storage
   let client = Client::default();
   let response = client
       .post("https://api.web3.storage/upload")
       .header("authorization", format!("Bearer {}", web3_token))
       .send_json(&cipher_str)
       .await;

   let response_body_str = response.unwrap().body().await.unwrap();
   let response_body: Value = serde_json::from_slice(&response_body_str).unwrap();
   return Ok(response_body);
}