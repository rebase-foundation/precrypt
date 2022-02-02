use indicatif::ProgressBar;
use indicatif::ProgressStyle;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use umbral_pre::*;

struct EnChunkMessage {
   bytes: Vec<u8>,
   capsule: Capsule,
   index: usize,
}

struct DeChunkMessage {
   bytes: Vec<u8>,
   index: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct RecryptionKeys {
   owner_secret: Vec<u8>,
   capsules: Vec<Capsule>,
   chunk_size: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DecryptionKeys {
   owner_pubkey: PublicKey,
   capsules: Vec<Capsule>,
   translated_keys: Vec<Vec<u8>>,
   chunk_size: usize,
}

impl DecryptionKeys {
   fn next_keys(&mut self) -> std::io::Result<(Capsule, Vec<u8>)> {
      let capsule: Capsule = self.capsules.remove(0);
      let translated_key: Vec<u8> = self.translated_keys.remove(0);
      return Ok((capsule, translated_key.clone()));
   }
}

// WARN: These numbers must be evenly divisible
const MAX_MEM_BYTES: usize = 50000000; // Max amount you want to hold in memory at once
const THREADS: usize = 10; // Max number of threads you want to run
                           // chunk_size = MAX_MEM_BYTES / THREADS

pub fn precrypt(
   input_path: &OsStr,
   file_key: SecretKey,
   output_keys: &OsStr,
   output_file: &OsStr,
) -> std::io::Result<()> {
   let f = File::open(input_path)?;
   let file_size = f.metadata().unwrap().len();
   let mut batches_remaining = (file_size as f64 / MAX_MEM_BYTES as f64).ceil() as u64;
   let mut capsules: Vec<Capsule> = Vec::new();
   // Remove ouput file file if it exists
   if std::path::Path::new(output_file).exists() {
      std::fs::remove_file(output_file)?;
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
      let (batch_encrypted, batch_capsules) = precrypt_batch(&f, file_key.public_key())?;
      capsules.extend(batch_capsules);
      // Append encrypted chunks to file
      out.write(&batch_encrypted)?;
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
      chunk_size: (MAX_MEM_BYTES / THREADS) + 40,
   };
   std::fs::write(
      output_keys,
      serde_json::to_string(&recryption_keys).unwrap(),
   )?;
   return Ok(());
}

fn precrypt_batch(f: &File, pubkey: PublicKey) -> std::io::Result<(Vec<u8>, Vec<Capsule>)> {
   let (tx, rx) = mpsc::channel();
   for x in 0..THREADS {
      let mut buffer = Vec::new();
      f.take((MAX_MEM_BYTES / THREADS) as u64)
         .read_to_end(&mut buffer)?;
      if buffer.len() == 0 {
         break;
      }
      let txc = tx.clone();
      thread::spawn(move || {
         let (capsule, cipher_chunk) = encrypt(&pubkey, &buffer).unwrap();
         let message = EnChunkMessage {
            bytes: cipher_chunk.to_vec(),
            index: x,
            capsule: capsule,
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
   let mut capsules: Vec<Capsule> = Vec::new();
   for m in messages {
      batch.extend(m.bytes);
      capsules.push(m.capsule);
   }
   return Ok((batch, capsules));
}

pub fn recrypt(
   recryption_keys: RecryptionKeys,
   receiver_public: PublicKey,
   output_path: &OsStr,
) -> std::io::Result<()> {
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
   for capsule in recryption_keys.capsules {
      let translated_key = reencrypt(&capsule, translation_key.clone());
      translated_keys.push(translated_key.to_array().to_vec());
   }

   let decryption_keys = DecryptionKeys {
      owner_pubkey: owner_secret.public_key(),
      capsules: capsules,
      translated_keys: translated_keys,
      chunk_size: recryption_keys.chunk_size,
   };

   std::fs::write(
      output_path,
      serde_json::to_string(&decryption_keys).unwrap(),
   )
   .unwrap();
   return Ok(());
}

pub fn decrypt(
   input_path: &OsStr,
   output_file: &OsStr,
   receiver_key: SecretKey,
   decryption_keys: &mut DecryptionKeys,
) -> std::io::Result<()> {
   let mut batches_remaining =
      (decryption_keys.capsules.len() as f64 / THREADS as f64).ceil() as u64;
   println!("Batches needed: {}", batches_remaining);
   // Read input file
   let f = File::open(input_path)?;
   // Remove ouput file file if it exists
   if std::path::Path::new(output_file).exists() {
      std::fs::remove_file(output_file)?;
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
      let batch_decrypted = decrypt_batch(&f, &receiver_key, decryption_keys)?;
      // Append encrypted chunks to file
      out.write(&batch_decrypted)?;
      batches_remaining -= 1;
      bar.inc(1);
   }
   bar.finish_and_clear();
   return Ok(());
}

fn decrypt_batch(
   f: &File,
   receiver_key: &SecretKey,
   decryption_keys: &mut DecryptionKeys,
) -> std::io::Result<Vec<u8>> {
   let (tx, rx) = mpsc::channel();
   for x in 0..THREADS {
      let mut buffer = Vec::new();
      f.take(decryption_keys.chunk_size as u64)
         .read_to_end(&mut buffer)?;
      if buffer.len() == 0 {
         break;
      }

      // Make clones of variables the thread will use
      let txc = tx.clone();
      let (capsule, translated_key_vec) = decryption_keys.next_keys()?;
      let translated_key = VerifiedCapsuleFrag::from_verified_bytes(translated_key_vec).unwrap();
      let receiver_key = receiver_key.clone();
      let owner_pubkey = decryption_keys.owner_pubkey.clone();
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
   return Ok(batch);
}
