use std::path::Path;

// 1) No folder & No result: Not found
// 2) Folder & No contents: Uploading
// 3) Folder & Plaintext: Encrypting
// 4) Folder & Cipher: Uploading to IFPS
// 5) No folder & Result: Ready
pub enum StoreStatus {
   Uploading,
   Encrypting,
   UploadingIPFS,
   Ready,
   NotFound,
}
pub fn store_status(uuid: String) -> StoreStatus {
   let has_folder = Path::new("request_results").is_dir();
   if has_folder {
      let has_plaintext = Path::new(&format!("{}/plaintext.bin", &uuid)).is_file();
      let has_cipher = Path::new(&format!("{}/cipher.bin", &uuid)).is_file();
      if !has_cipher && !has_plaintext {
         return StoreStatus::Uploading;
      } else if !has_cipher && has_plaintext {
         return StoreStatus::Encrypting;
      } else if has_cipher && has_plaintext {
         return StoreStatus::UploadingIPFS;
      }
   }
   let has_result = Path::new(&format!("store_keys/{}.txt", &uuid)).is_file(); // TODO: update for file ending
   if has_result {
      return StoreStatus::Ready;
   }
   return StoreStatus::NotFound;
}
