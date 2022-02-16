use actix_web::client::Client;
use actix_web::web::Bytes;
use glob::glob;
use serde_json::json;
use serde_json::Value;
use std::fs;

use crate::util::command::run_command;
use crate::util::get_secrets::get_secrets;
use crate::util::path_builder::{build_path, PathBuilder};

const MAX_RETRIES: i64 = 3;

pub async fn upload_cars(uuid: &String) -> String {
   let root_cid = get_root_cid(&uuid);

   let pattern = build_path(PathBuilder::CarPattern, &uuid);
   for entry in glob(&pattern).expect("Failed to read glob pattern") {
      let path = entry.unwrap();
      // Get cipher bytes for upload
      let cipher_bytes = fs::read(&path).unwrap();
      let body = actix_web::web::Bytes::from_iter(cipher_bytes);
      println!("Uploading: {:?}", &path);
      let json = web3_post(body).await.unwrap();
      if root_cid.eq(&json["cid"].to_string()) {
         let msg = format!(
            "Received CID different from root: {}",
            json["cid"].to_string()
         );
         panic!("{}", msg);
      }
   }
   return root_cid;
}

pub async fn web3_post(body: Bytes) -> Result<Value, Value> {
   let (_, web3_token) = get_secrets().unwrap();
   let mut retries = 0;
   let mut json: Option<Value> = None;
   let mut error: Option<Value> = None;
   let client = Client::default();
   while retries < MAX_RETRIES {
      if retries > 0 {println!("Retrying...");}
      let resp = client
         .post("https://api.web3.storage/car")
         .header("authorization", format!("Bearer {}", web3_token))
         .header("Content-Type", "application.car")
         .timeout(std::time::Duration::new(120, 0))
         .send_body(body.clone()).await;
      if resp.is_ok() {
         let mut file_response = resp.unwrap();
         let status = file_response.status().as_u16();
         let file_response_bytes = file_response.body().await.unwrap();
         let response_json: Value = serde_json::from_slice(&file_response_bytes).unwrap();
         if status == 200 {
            println!("[{}] Request successful: {}", status, response_json);
            json = Some(response_json);
            break;
         } else {
            println!(
               "[{}] Request returned error code: {}",
               status, response_json
            );
            error = Some(response_json);
            retries += 1;
         }
      } else {
         let send_error = resp.unwrap_err();
         println!("Error sending request: {}", send_error);
         error = Some(json!({
            "message": format!("{:?}", send_error)
         }));
         retries += 1;
      }
   }
   if json.is_some() {
      return Ok(json.unwrap());
   } else {
      return Err(error.unwrap());
   }
}

pub fn get_root_cid(uuid: &String) -> String {
   // Get root cid for the cars
   let cipher_car_path = build_path(PathBuilder::CipherCar, &uuid);
   let output = run_command(format!("ipfs-car --list-roots {}", cipher_car_path)).unwrap();
   return std::str::from_utf8(&output.stdout)
      .unwrap()
      .replace("\n", "");
}
