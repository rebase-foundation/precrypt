use std::env;

pub enum PathBuilder {
   TaskDir,
   Plaintext,
   CipherCar,
   CarPattern,
   Cipher,
   Car,
   StoreResultDir,
   StoreResult,
   RequestResultDir,
   RequestResult
}

pub fn build_path(path: PathBuilder, uuid: &String) -> String {
   let rel_path = match path {
      PathBuilder::TaskDir => {
         format!("{}", uuid)
      }
      PathBuilder::Plaintext => {
         format!("{}/plaintext.bin", uuid)
      }
      PathBuilder::CipherCar => {
         format!("{}/cipher.car", uuid)
      }
      PathBuilder::Cipher => {
         format!("{}/cipher.bin", uuid)
      }
      PathBuilder::Car => {
         format!("{}/cipher-0.car", uuid)
      }
      PathBuilder::CarPattern => {
         format!("{}/cipher-*.car", uuid)
      }
      PathBuilder::StoreResult => {
         format!("store_results/{}.json", uuid)
      }
      PathBuilder::StoreResultDir => {
         "store_results".to_string()
      }
      PathBuilder::RequestResult => {
         format!("request_results/{}.bin", uuid)
      }
      PathBuilder::RequestResultDir => {
         "request_results".to_string()
      }
   };
   let volume_dir = env::var("DATA").unwrap();
   let full_path = format!("{}/{}", volume_dir, rel_path);
   return full_path;
}
