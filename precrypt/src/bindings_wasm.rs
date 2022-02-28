//! Type wrappers for WASM bindings.
use crate as precrypt;
use generic_array::GenericArray;
use umbral_pre::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub struct RecryptionKeys(precrypt::RecryptionKeys);

#[wasm_bindgen]
pub struct PrecryptBytesResult(precrypt::PrecryptBytesResult);

#[wasm_bindgen]
pub fn precrypt_bytes(
    bytes: Vec<u8>,
    wasm_public_key: umbral_pre::bindings_wasm::PublicKey,
) -> PrecryptBytesResult {
    let public_key = PublicKey::from_array(&GenericArray::from_iter(wasm_public_key.to_bytes().to_vec())).unwrap();
    return PrecryptBytesResult(precrypt::precrypt_bytes_async(bytes, public_key));
}

#[wasm_bindgen]
pub struct DecryptionKeys(precrypt::DecryptionKeys);

#[wasm_bindgen]
pub fn recrypt_keys(
    wasm_recryption_keys: RecryptionKeys,
    wasm_receiver_public: umbral_pre::bindings_wasm::PublicKey,
) -> DecryptionKeys {
    let receiver_public = PublicKey::from_array(&GenericArray::from_iter(
        wasm_receiver_public.to_bytes().to_vec(),
    ))
    .unwrap();
    return DecryptionKeys(precrypt::recrypt_keys(wasm_recryption_keys.0, receiver_public));
}

pub fn decrypt_file(
    input_path: &str,
    output_file: &str,
    wasm_receiver_key: umbral_pre::bindings_wasm::SecretKey,
    wasm_decryption_keys: &mut DecryptionKeys,
    threads: usize,
) {
    let file_key = umbral_pre::SecretKey::from_array(&GenericArray::from_iter(
        wasm_receiver_key.to_secret_bytes().to_vec(),
    ))
    .unwrap();
    precrypt::decrypt_file(input_path, output_file, file_key, &mut wasm_decryption_keys.0, threads);
}
