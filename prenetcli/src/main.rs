use clap::Arg;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Result;
use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use clap::{App, AppSettings};
use umbral_pre::*;

#[derive(Serialize, Deserialize)]
struct Keypair {
    public_key: PublicKey,
    secret_key: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Translator {
    owner_pubkey: PublicKey,
    capsule: Capsule,
    translation_key: Vec<u8>,
}

fn parse_keypair_file(path: &OsStr) -> std::io::Result<SecretKey> {
    let file = File::open(path)?;
    let keypair_json: Keypair = serde_json::from_reader(BufReader::new(file))?;
    return Ok(SecretKey::from_bytes(keypair_json.secret_key).unwrap());
}

fn main() -> std::io::Result<()> {
    let matches = App::new("prenet")
        .about("Cli for pre-network")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("encrypt").about("Encrypts file").args([
                Arg::new("input")
                    .allow_invalid_utf8(true)
                    .help("Path of the file to be encrypted")
                    .required(true),
                Arg::new("owner_keypair")
                    .allow_invalid_utf8(true)
                    .help("Path of the keypair to encrypt the file with")
                    .required(true),
                Arg::new("output")
                    .allow_invalid_utf8(true)
                    .help("Output path for the new encrypted file")
                    .required(true),
            ]),
        )
        .subcommand(
            App::new("recrypt")
                .about("Translates encryption key to a new pubkey")
                .args([
                    Arg::new("owner_keypair")
                        .allow_invalid_utf8(true)
                        .help("Path of the keypair to encrypt the file with")
                        .required(true),
                    Arg::new("receiver_pubkey")
                        .help("Path of the keypair to encrypt the file with")
                        .required(true),
                    Arg::new("output")
                        .allow_invalid_utf8(true)
                        .help("Output path for translator json")
                        .required(true),
                ]),
        )
        .subcommand(
            App::new("decrypt")
                .about("Decrypts the input file using translation and receiver keys")
                .args([
                    Arg::new("input")
                        .allow_invalid_utf8(true)
                        .help("Path of the file to be encrypted")
                        .required(true),
                    Arg::new("translator")
                        .allow_invalid_utf8(true)
                        .help("Path of the translator json file")
                        .required(true),
                    Arg::new("receiver_keypair")
                        .allow_invalid_utf8(true)
                        .help("Path of the keypair to encrypt the file with")
                        .required(true),
                    Arg::new("output")
                        .allow_invalid_utf8(true)
                        .help("Output path for the new encrypted file")
                        .required(true),
                ]),
        )
        .subcommand(
            App::new("keygen").about("Generates new keypair").arg(
                Arg::new("output")
                    .allow_invalid_utf8(true)
                    .help("Output path of where to store the keypair")
                    .required(true),
            ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("encrypt", sub_matches)) => {
            // Read the input file to a buffer
            let input_path = sub_matches.value_of_os("input").unwrap();
            let plaintext = fs::read(input_path)?;
            // Read the keypair file
            let keypair_path = sub_matches.value_of_os("owner_keypair").unwrap();
            let file_secret: SecretKey = parse_keypair_file(&keypair_path)?;
            let file_pub = file_secret.public_key();

            // Use that key to encrypt it
            let (capsule, ciphertext) = encrypt(&file_pub, &plaintext).unwrap();
            std::fs::write("capsule.json", serde_json::to_string(&capsule).unwrap()).unwrap();
            let output_path = sub_matches.value_of_os("output").unwrap();
            std::fs::write(output_path, bincode::serialize(&ciphertext).unwrap()).unwrap();
            Ok(())
        }
        Some(("recrypt", sub_matches)) => {
            // Read capsule
            let file = File::open("capsule.json")?;
            let capsule: Capsule = serde_json::from_reader(BufReader::new(file))?;

            // Read owner key from file
            let keypair_path = sub_matches.value_of_os("owner_keypair").unwrap();
            let file_secret: SecretKey = parse_keypair_file(&keypair_path)?;

            // Read receiver pubkey from argunment
            let receiver_public_str = sub_matches.value_of("receiver_pubkey").unwrap();
            let receiver_public_json = format!("\"{}\"", receiver_public_str); // Turns raw string into json string
            let receiver_public: PublicKey = serde_json::from_str(&receiver_public_json)?;

            // Fragmentation/verification is not used because we aren't using proxies
            let translation_key = generate_kfrags(
                &file_secret,
                &receiver_public,
                &Signer::new(SecretKey::random()),
                1,
                1,
                false,
                false,
            )[0]
            .clone();

            let translated_key = reencrypt(&capsule, translation_key);
            let translator = Translator {
                owner_pubkey: file_secret.public_key(),
                capsule: capsule,
                translation_key: translated_key.to_array().to_vec(),
            };
            let output_path = sub_matches.value_of_os("output").unwrap();
            std::fs::write(output_path, serde_json::to_string(&translator).unwrap()).unwrap();
            Ok(())
        }
        Some(("decrypt", sub_matches)) => {
            // Read cipher
            let input_path = sub_matches.value_of_os("input").unwrap();
            let reader = BufReader::new(File::open(input_path)?);
            let ciphertext: Box<[u8]> = bincode::deserialize_from(reader).unwrap();
            
            // Read recrypt response
            let translator_path = sub_matches.value_of_os("translator").unwrap();
            let translator_array = std::fs::read(translator_path).unwrap();
            let translator: Translator = serde_json::from_slice(&translator_array)?;
            let translation_key =
                VerifiedCapsuleFrag::from_verified_bytes(translator.translation_key).unwrap();

            // Read receiver secret
            let keypair_path = sub_matches.value_of_os("receiver_keypair").unwrap();
            let receiver_secret: SecretKey = parse_keypair_file(&keypair_path)?;

            // Code the receiver runs
            let plaintext_receiver = decrypt_reencrypted(
                &receiver_secret,
                &translator.owner_pubkey,
                &translator.capsule,
                [translation_key],
                &ciphertext,
            )
            .unwrap();
            let plain_vec = plaintext_receiver.to_vec();
            println!("{}", String::from_utf8(plain_vec).unwrap());
            Ok(())
        }
        Some(("keygen", sub_matches)) => {
            let output_path = sub_matches.value_of_os("output").unwrap();
            let keypair = SecretKey::random();

            // Parse secret as array
            let secret_box = keypair.to_secret_array();
            let secret_array = secret_box.as_secret().to_vec();

            let keypair = Keypair {
                public_key: keypair.public_key(),
                secret_key: secret_array.to_owned(),
            };
            std::fs::write(output_path, serde_json::to_string(&keypair).unwrap()).unwrap();
            Ok(())
        }
        _ => unreachable!(),
    }
}
