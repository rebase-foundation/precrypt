use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use serde::{Deserialize, Serialize};
use generic_array::GenericArray;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use umbral_pre::*;
use umbral_pre::DeserializableFromArray;

pub mod bindings_wasm;

struct EnChunkMessage {
   bytes: Vec<u8>,
   capsule: Vec<u8>,
   index: usize,
}

struct DeChunkMessage {
   bytes: Vec<u8>,
   index: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecryptionKeys {
   owner_secret: Vec<u8>,
   capsules: Vec<Vec<u8>>,
   chunk_size: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DecryptionKeys {
   owner_pubkey: Vec<u8>,
   capsules: Vec<Vec<u8>>,
   translated_keys: Vec<Vec<u8>>,
   chunk_size: usize,
}

impl DecryptionKeys {
   fn next_keys(&mut self) -> (Vec<u8>, Vec<u8>) {
      let capsule: Vec<u8> = self.capsules.remove(0);
      let translated_key: Vec<u8> = self.translated_keys.remove(0);
      return (capsule, translated_key.clone());
   }
}

pub fn precrypt_file(
   input_path: &str,
   file_key: SecretKey,
   output_file: &str,
   threads: usize,
   memory_size: usize,
) -> RecryptionKeys {
   if memory_size % threads != 0 {
      panic!("'memory_size' must be evenly divisible by 'threads'")
   }

   let f = File::open(input_path).unwrap();
   let file_size = f.metadata().unwrap().len();
   let mut batches_remaining = (file_size as f64 / memory_size as f64).ceil() as u64;
   let mut capsules: Vec<Vec<u8>> = Vec::new();
   // Remove output file if it exists
   if std::path::Path::new(output_file).exists() {
      std::fs::remove_file(output_file).unwrap();
   }
   let mut out = OpenOptions::new()
      .write(true)
      .append(true)
      .create_new(true)
      .open(output_file)
      .unwrap();

   println!("Encrypting file: {:?}", input_path);
   let bar = ProgressBar::new(batches_remaining);
   bar.set_style(
      ProgressStyle::default_bar()
         .template("{eta} [{bar:40.cyan/blue}] {percent}%")
         .progress_chars("=>-"),
   );
   while batches_remaining > 0 {
      let (batch_encrypted, batch_capsules) =
         precrypt_batch(&f, file_key.public_key(), threads, memory_size);
      capsules.extend(batch_capsules);
      // Append encrypted chunks to file
      out.write(&batch_encrypted).unwrap();
      batches_remaining -= 1;
      bar.inc(1);
   }
   bar.finish_and_clear();

   // Write out recryption keys
   let secret_box = file_key.to_secret_array();
   let secret_array = secret_box.as_secret().to_vec();
   let recryption_keys = RecryptionKeys {
      owner_secret: secret_array,
      capsules: capsules,
      chunk_size: (memory_size / threads) + 40,
   };
   return recryption_keys;
}

fn precrypt_batch(
   f: &File,
   pubkey: PublicKey,
   threads: usize,
   memory_size: usize,
) -> (Vec<u8>, Vec<Vec<u8>>) {
   let (tx, rx) = mpsc::channel();
   for x in 0..threads {
      let mut buffer = Vec::new();
      f.take((memory_size / threads) as u64)
         .read_to_end(&mut buffer)
         .unwrap();
      if buffer.len() == 0 {
         break;
      }
      let txc = tx.clone();
      thread::spawn(move || {
         let (capsule, cipher_chunk) = umbral_pre::encrypt(&pubkey, &buffer).unwrap();
         let message = EnChunkMessage {
            bytes: cipher_chunk.to_vec(),
            index: x,
            capsule: capsule.to_array().to_vec(),
         };
         txc.send(message).unwrap();
      });
   }

   // drop tx manually, to ensure that only senders in spawned threads are still in use
   drop(tx);

   // Add all the chunk messages to a vector
   let mut messages: Vec<EnChunkMessage> = Vec::new();
   for message in rx {
      messages.push(message);
   }
   // Sort messages by index (order in input file)
   messages.sort_by(|a, b| a.index.cmp(&b.index));
   // Combine messages into a batch
   let mut batch: Vec<u8> = Vec::new();
   let mut capsules: Vec<Vec<u8>> = Vec::new();
   for m in messages {
      batch.extend(m.bytes);
      capsules.push(m.capsule);
   }
   return (batch, capsules);
}

pub fn recrypt_keys(recryption_keys: RecryptionKeys, receiver_public: PublicKey) -> DecryptionKeys {
   // Fragmentation/verification is not used because we aren't using proxies
   let owner_secret: SecretKey = SecretKey::from_bytes(recryption_keys.owner_secret).unwrap();
   let translation_key = generate_kfrags(
      &owner_secret,
      &receiver_public,
      &Signer::new(SecretKey::random()),
      1,
      1,
      false,
      false,
   )[0]
      .clone();

   let mut translated_keys: Vec<Vec<u8>> = Vec::new();
   let capsules = recryption_keys.capsules.clone();
   for capsule_vec in recryption_keys.capsules {
      let capsule = Capsule::from_array(&GenericArray::from_iter(capsule_vec)).unwrap();
      let translated_key = reencrypt(&capsule, translation_key.clone());
      translated_keys.push(translated_key.to_array().to_vec());
   }

   let decryption_keys = DecryptionKeys {
      owner_pubkey: owner_secret.public_key().to_array().to_vec(),
      capsules: capsules,
      translated_keys: translated_keys,
      chunk_size: recryption_keys.chunk_size,
   };
   return decryption_keys;
}

pub fn decrypt_file(
   input_path: &str,
   output_file: &str,
   receiver_key: SecretKey,
   decryption_keys: &mut DecryptionKeys,
   threads: usize,
) {
   let mut batches_remaining =
      (decryption_keys.capsules.len() as f64 / threads as f64).ceil() as u64;
   println!("Batches needed: {}", batches_remaining);
   // Read input file
   let f = File::open(input_path).unwrap();
   // Remove output file file if it exists
   if std::path::Path::new(output_file).exists() {
      std::fs::remove_file(output_file).unwrap();
   }
   let mut out = OpenOptions::new()
      .write(true)
      .append(true)
      .create_new(true)
      .open(output_file)
      .unwrap();

   println!("Decrypting file: {:?}", input_path);
   let bar = ProgressBar::new(batches_remaining);
   bar.set_style(
      ProgressStyle::default_bar()
         .template("{eta} [{bar:40.cyan/blue}] {percent}%")
         .progress_chars("=>-"),
   );
   while batches_remaining > 0 {
      let batch_decrypted = decrypt_batch(&f, &receiver_key, decryption_keys, threads);
      // Append encrypted chunks to file
      out.write(&batch_decrypted).unwrap();
      batches_remaining -= 1;
      bar.inc(1);
   }
   bar.finish_and_clear();
}

fn decrypt_batch(
   f: &File,
   receiver_key: &SecretKey,
   decryption_keys: &mut DecryptionKeys,
   threads: usize,
) -> Vec<u8> {
   let (tx, rx) = mpsc::channel();
   for x in 0..threads {
      let mut buffer = Vec::new();
      f.take(decryption_keys.chunk_size as u64)
         .read_to_end(&mut buffer)
         .unwrap();
      if buffer.len() == 0 {
         break;
      }

      // Make clones of variables the thread will use
      let txc = tx.clone();
      let (capsule_vec, translated_key_vec) = decryption_keys.next_keys();
      let translated_key = VerifiedCapsuleFrag::from_verified_bytes(translated_key_vec).unwrap();
      let receiver_key = receiver_key.clone();
      let owner_pubkey_vec = decryption_keys.owner_pubkey.clone();
      let owner_pubkey = PublicKey::from_array(&GenericArray::from_iter(owner_pubkey_vec)).unwrap();
      let capsule = Capsule::from_array(&GenericArray::from_iter(capsule_vec)).unwrap();
      thread::spawn(move || {
         // Decrypt the cipher
         let plaintext = decrypt_reencrypted(
            &receiver_key,
            &owner_pubkey,
            &capsule,
            [translated_key],
            &buffer,
         )
         .unwrap();
         txc.send(DeChunkMessage {
            bytes: plaintext.to_vec(),
            index: x,
         })
         .unwrap();
      });
   }

   // drop tx manually, to ensure that only senders in spawned threads are still in use
   drop(tx);

   // Add all the chunk messages to a vector
   let mut messages: Vec<DeChunkMessage> = Vec::new();
   for message in rx {
      messages.push(message);
   }
   // Sort messages by index (order in input file)
   messages.sort_by(|a, b| a.index.cmp(&b.index));
   // Combine messages into a batch
   let mut batch: Vec<u8> = Vec::new();
   for m in messages {
      batch.extend(m.bytes);
   }
   return batch;
}
