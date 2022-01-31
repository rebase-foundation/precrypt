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
    public_key: String,
    secret_key: Vec<u8>,
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
                    Arg::new("input")
                        .allow_invalid_utf8(true)
                        .help("Path of the file to be encrypted")
                        .required(true),
                    Arg::new("owner_keypair")
                        .allow_invalid_utf8(true)
                        .help("Path of the keypair to encrypt the file with")
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
            App::new("decrypt")
                .about("Decrypts the input file using translation and receiver keys")
                .args([
                    Arg::new("input")
                        .allow_invalid_utf8(true)
                        .help("Path of the file to be encrypted")
                        .required(true),
                    Arg::new("owner_pubkey")
                        .help("Public key of the file owner")
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
            // Read the input file to a string
            let input_path = sub_matches.value_of_os("input").unwrap();
            // let file = File::open(input_path)?;
            // let mut buf_reader = BufReader::new(file);
            let plaintext = fs::read(input_path)?;
            // buf_reader.read_to_string(&mut plaintext)?;
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

            // Read cipher
            // let reader = BufReader::new(File::open("cipher.bin")?);
            // let ciphertext: Box<u8> = bincode::deserialize_from(reader).unwrap();

            // Read keys
            let keypair_path = sub_matches.value_of_os("owner_keypair").unwrap();
            let file_secret: SecretKey = parse_keypair_file(&keypair_path)?;
            let keypair_path = sub_matches.value_of_os("receiver_keypair").unwrap();
            let receiver_secret: SecretKey = parse_keypair_file(&keypair_path)?;
            let receiver_public = receiver_secret.public_key();

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
            std::fs::write("translate.bin", translated_key.to_array()).unwrap();
            Ok(())
        }
        Some(("decrypt", sub_matches)) => {
            // Read capsule
            let file = File::open("capsule.json")?;
            let capsule: Capsule = serde_json::from_reader(BufReader::new(file))?;

            // Read cipher
            let input_path = sub_matches.value_of_os("input").unwrap();
            let reader = BufReader::new(File::open(input_path)?);
            let ciphertext: Box<[u8]> = bincode::deserialize_from(reader).unwrap();

            // Read translation key
            let translated_array = std::fs::read("translate.bin").unwrap();
            let translated_key: VerifiedCapsuleFrag =
                VerifiedCapsuleFrag::from_verified_bytes(translated_array).unwrap();

            // Read file pub
            let file_pub_str = sub_matches.value_of("owner_pubkey").unwrap();
            let file_pub: PublicKey = serde_json::from_str(file_pub_str)?;

            // Read receiver secret
            let keypair_path = sub_matches.value_of_os("receiver_keypair").unwrap();
            let receiver_secret: SecretKey = parse_keypair_file(&keypair_path)?;

            // Code the receiver runs
            let plaintext_receiver = decrypt_reencrypted(
                &receiver_secret,
                &file_pub,
                &capsule,
                [translated_key],
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
                public_key: serde_json::to_string(&keypair.public_key())?,
                secret_key: secret_array.to_owned(),
            };
            std::fs::write(output_path, serde_json::to_string(&keypair).unwrap()).unwrap();
            Ok(())
        }
        _ => unreachable!(),
    }
}
