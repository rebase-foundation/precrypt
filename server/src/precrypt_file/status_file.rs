use std::path::Path;

// 1) No folder & No result: Not found
// 2) Folder & Plaintext: Encrypting
// 3) Folder & Cipher: Processing
// 4) Folder & Cipher & Car: Uploading to IFPS
// 5) No folder & Result: Ready
pub enum StoreStatus {
   EncryptingPlaintext,
   ProcessingCipher,
   UploadingCipher,
   Ready,
   NotFound
}
pub fn store_status(uuid: String) -> StoreStatus {
   let has_result = Path::new(&format!("store_results/{}.json", &uuid)).is_file();
   if has_result {
      return StoreStatus::Ready;
   }

   let has_folder = Path::new(&format!("{}", &uuid)).is_dir();
   if has_folder {
      let has_cipher = Path::new(&format!("{}/cipher.bin", &uuid)).is_file();
      let has_car = Path::new(&format!("{}/cipher-0.car", &uuid)).is_file();
      if !has_cipher {
         return StoreStatus::EncryptingPlaintext;
      } else if has_cipher && !has_car {
         return StoreStatus::ProcessingCipher;
      } else if has_cipher && has_car {
         return StoreStatus::UploadingCipher;
      }
   }
   return StoreStatus::NotFound;
}

// 1) No folder & No result: Not found
// 2) Folder & No contents: Downloading from IFPS
// 3) Folder & Cipher: Decrypting
// 4) No folder and result: Ready
pub enum RequestStatus {
   DownloadingCipher,
   UnpackingCipher,
   DecryptingCipher,
   Ready,
   NotFound,
}
pub fn request_status(uuid: String) -> RequestStatus {
   let has_folder = Path::new(&format!("{}", &uuid)).is_dir();
   if has_folder {
      let has_car = Path::new(&format!("{}/cipher.car", &uuid)).is_file();
      let has_cipher = Path::new(&format!("{}/cipher.txt", &uuid)).is_file();
      if !has_car {
         return RequestStatus::DownloadingCipher;
      } else if has_car && !has_cipher {
         return RequestStatus::UnpackingCipher;
      } else if has_car && has_cipher {
         return RequestStatus::DecryptingCipher;
      }
   }
   let has_result = Path::new(&format!("request_results/{}.txt", &uuid)).is_file(); // TODO: update for file ending
   if has_result {
      return RequestStatus::Ready;
   }
   return RequestStatus::NotFound;
}
