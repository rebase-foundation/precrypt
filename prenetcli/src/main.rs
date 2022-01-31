use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::io::Write; // bring trait into scope
use std::path::PathBuf;

use clap::{arg, App, AppSettings};
use umbral_pre::*;

fn main() -> std::io::Result<()> {
    let matches = App::new("prenet")
        .about("Cli for pre-network")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("encrypt")
                .about("Encrypts file")
                .arg(arg!(<PATH> ... "Stuff to encrypt").allow_invalid_utf8(true)),
        )
        .subcommand(
            App::new("recrypt")
                .about("Translates encryption key to a new pubkey")
                .arg(arg!(<PATH> ... "Stuff to encrypt").allow_invalid_utf8(true)),
        )
        .subcommand(
            App::new("decrypt")
                .about("Translates encryption key to a new pubkey")
                .arg(arg!(<PATH> ... "Stuff to encrypt").allow_invalid_utf8(true)),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("encrypt", sub_matches)) => {
            let paths = sub_matches
                .values_of_os("PATH")
                .unwrap_or_default()
                .map(PathBuf::from)
                .collect::<Vec<_>>();
            for path in paths {
                let file = File::open(path)?;
                let mut buf_reader = BufReader::new(file);
                let mut plaintext = String::new();
                buf_reader.read_to_string(&mut plaintext)?;
                // TODO: SEPERATE INTO COMMANDS BY PERSONA
                // Create a new random secret for this file
                let file_secret = SecretKey::random();
                // println!("File Secret: {}")
                let file_pub = file_secret.public_key();

                // Use that key to encrypt it
                let (capsule, ciphertext) = encrypt(&file_pub, plaintext.as_bytes()).unwrap();
                std::fs::write("capsule.bin", bincode::serialize(&capsule).unwrap()).unwrap();
                std::fs::write("cipher.bin", bincode::serialize(&ciphertext).unwrap()).unwrap();
                
                let file_secret_box = file_secret.to_secret_array();
                let secret_array = file_secret_box.as_secret();
                std::fs::write("sec.key", secret_array).unwrap();
                std::fs::write("pub.key", bincode::serialize(&file_pub).unwrap()).unwrap();
            }
            Ok(())
        }
        Some(("recrypt", sub_matches)) => {
            // Read capsule
            let reader = BufReader::new(File::open("capsule.bin")?);
            let capsule: Capsule = bincode::deserialize_from(reader).unwrap();

            // Read cipher
            // let reader = BufReader::new(File::open("cipher.bin")?);
            // let ciphertext: Box<u8> = bincode::deserialize_from(reader).unwrap();

            // Read key
            let secret_array: Vec<u8> = std::fs::read("sec.key").unwrap();
            let file_secret: SecretKey = SecretKey::from_bytes(&secret_array).unwrap();

            let buyer_secret = SecretKey::random();
            let buyer_secret_box = buyer_secret.to_secret_array();
            let buyer_secret_array = buyer_secret_box.as_secret();
            std::fs::write("buyer_sec.key", buyer_secret_array).unwrap();
            let buyer_public = buyer_secret.public_key();

            // Fragmentation/verification is not used because we aren't using proxies
            let translation_key = generate_kfrags(
                &file_secret,
                &buyer_public,
                &Signer::new(SecretKey::random()),
                1,
                1,
                false,
                false,
            )[0]
            .clone();
            let translated_key = reencrypt(&capsule, translation_key);
            std::fs::write("translate.bin", translated_key.to_array()).unwrap();
            Ok(())
        }
        Some(("decrypt", sub_matches)) => {
            // Read capsule
            let reader = BufReader::new(File::open("capsule.bin")?);
            let capsule: Capsule = bincode::deserialize_from(reader).unwrap();

            // Read cipher
            let reader = BufReader::new(File::open("cipher.bin")?);
            let ciphertext: Box<[u8]> = bincode::deserialize_from(reader).unwrap();

            // Read translation key
            let translated_array = std::fs::read("translate.bin").unwrap();
            let translated_key: VerifiedCapsuleFrag = VerifiedCapsuleFrag::from_verified_bytes(translated_array).unwrap();
            
            // Read file pub
            let reader = BufReader::new(File::open("pub.key")?);
            let file_pub: PublicKey = bincode::deserialize_from(reader).unwrap();

            // Read buyer secret
            let secret_array: Vec<u8> = std::fs::read("buyer_sec.key").unwrap();
            let buyer_secret: SecretKey = SecretKey::from_bytes(&secret_array).unwrap();


            // Code the buyer runs
            let plaintext_buyer = decrypt_reencrypted(
                &buyer_secret,
                &file_pub,
                &capsule,
                [translated_key],
                &ciphertext,
            )
            .unwrap();
        
            let plain_vec = plaintext_buyer.to_vec();
            println!("{}", String::from_utf8(plain_vec).unwrap());
            Ok(())
        }
        _ => unreachable!(),
    }
}
