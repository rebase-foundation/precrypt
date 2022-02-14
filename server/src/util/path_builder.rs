use glob::glob;

pub enum PathBuilder {
   TaskDir,
   Plaintext,
   RecryptKey,
   CipherCar,
   CarPattern,
   Cipher,
   Car,
   StoreResultDir,
   StoreResult,
   RequestResultDir,
   RequestResult,
   RequestResultGlob
}

pub fn build_path(path: PathBuilder, uuid: &String) -> String {
   match path {
      PathBuilder::TaskDir => {
         return format!("{}", uuid);
      }
      PathBuilder::Plaintext => {
         return format!("{}/plaintext.bin", uuid);
      }
      PathBuilder::RecryptKey => {
         return format!("{}/recrypt.json", uuid);
      }
      PathBuilder::CipherCar => {
         return format!("{}/cipher.car", uuid);
      }
      PathBuilder::Cipher => {
         return format!("{}/cipher.bin", uuid);
      }
      PathBuilder::Car => {
         return format!("{}/cipher-0.car", uuid);
      }
      PathBuilder::CarPattern => {
         return format!("{}/cipher-*.car", uuid);
      }
      PathBuilder::StoreResult => {
         return format!("store_results/{}.json", uuid);
      }
      PathBuilder::StoreResultDir => {
         return "store_results".to_string();
      }
      PathBuilder::RequestResult => {
         return format!("request_results/{}.bin", uuid);
      }
      PathBuilder::RequestResultDir => {
         return "request_results".to_string();
      }
      PathBuilder::RequestResultGlob => {
         let pattern = format!("request_results/{}.*", uuid);
         let path = glob(&pattern).unwrap().next().unwrap().unwrap();
         return path.to_str().unwrap().to_string();
      }
   }
}
