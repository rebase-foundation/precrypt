use crate::DecryptionKeys;
use crate::RecryptionKeys;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Read;
use std::io::Write;
use std::sync::mpsc;
use std::thread;
use umbral_pre::*;

use std::time::Instant;

struct EnChunkMessage {
   bytes: Vec<u8>,
   capsule: Capsule,
   index: usize,
}

struct DeChunkMessage {
   bytes: Vec<u8>,
   index: usize,
}

// WARN: These numbers must be evenly divisible
const MAX_MEM_BYTES: usize = 100000000; // Max amount you want to hold in memory at once
const THREADS: usize = 10; // Max number of threads you want to run
// chunk_size = MAX_MEM_BYTES / THREADS

pub fn encrypt_threaded(
   f: &File,
   file_key: SecretKey,
   output_keys: &OsStr,
   output_file: &OsStr
) -> std::io::Result<()> {
   let file_size = f.metadata().unwrap().len();
   println!("File size: {}", file_size);
   let mut batches_remaining = (file_size as f64 / MAX_MEM_BYTES as f64).ceil() as u64;
   println!("Batches needed: {}", batches_remaining);
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
   while batches_remaining > 0 {
      let start_time = Instant::now();

      let (batch_encrypted, batch_capsules) = encrypt_batch_threaded(f, file_key.public_key())?;
      let duration = start_time.elapsed();
      capsules.extend(batch_capsules);
      // Append encrypted chunks to file
      out.write(&batch_encrypted)?;
      batches_remaining -= 1;
      println!("{} batches remain ({:?})", batches_remaining, duration);
   }

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

fn encrypt_batch_threaded(f: &File, pubkey: PublicKey) -> std::io::Result<(Vec<u8>, Vec<Capsule>)> {
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

pub fn decrypt_threaded(
   f: &File,
   output_file: &OsStr,
   receiver_key: SecretKey,
   decryption_keys: &mut DecryptionKeys,
) -> std::io::Result<()> {
   let mut batches_remaining =  (decryption_keys.capsules.len() as f64 / THREADS as f64).ceil() as u64;
   println!("Batches needed: {}", batches_remaining);
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
   while batches_remaining > 0 {
      let start_time = Instant::now();

      let batch_decrypted = decrypt_batch_threaded(f, &receiver_key, decryption_keys)?;
      let duration = start_time.elapsed();
      // Append encrypted chunks to file
      out.write(&batch_decrypted)?;
      batches_remaining -= 1;
      println!("{} batches remain ({:?})", batches_remaining, duration);
   }
   return Ok(());
}

fn decrypt_batch_threaded(
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
