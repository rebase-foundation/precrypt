use std::path::Path;
use crate::util::path_builder::{build_path, PathBuilder};

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
   let store_result_path = build_path(PathBuilder::StoreResult, &uuid);
   let has_result = Path::new(&store_result_path).is_file();
   if has_result {
      return StoreStatus::Ready;
   }

   let has_folder = Path::new(&build_path(PathBuilder::TaskDir, &uuid)).is_dir();
   if has_folder {
      let cipher_path = build_path(PathBuilder::Cipher, &uuid);
      let has_cipher = Path::new(&cipher_path).is_file();
      let car_path = build_path(PathBuilder::Car, &uuid);
      let has_car = Path::new(&car_path).is_file();
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
   let has_folder = Path::new(&build_path(PathBuilder::TaskDir, &uuid)).is_dir();
   if has_folder {
      let cipher_path = build_path(PathBuilder::Cipher, &uuid);
      let has_cipher = Path::new(&cipher_path).is_file();
      let car_path = build_path(PathBuilder::Car, &uuid);
      let has_car = Path::new(&car_path).is_file();
      if !has_car {
         return RequestStatus::DownloadingCipher;
      } else if has_car && !has_cipher {
         return RequestStatus::UnpackingCipher;
      } else if has_car && has_cipher {
         return RequestStatus::DecryptingCipher;
      }
   }
   let path = build_path(PathBuilder::RequestResultGlob, &uuid);
   let has_result = Path::new(&path).is_file();
   if has_result {
      return RequestStatus::Ready;
   }
   return RequestStatus::NotFound;
}
