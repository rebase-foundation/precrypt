use crate::pre_threaded::decrypt_threaded;
use clap::Arg;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;

use clap::{App, AppSettings};
use umbral_pre::*;

mod pre_threaded;

#[derive(Serialize, Deserialize, Clone)]
struct Keypair {
    public_key: PublicKey,
    secret_key: Vec<u8>,
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
        return Ok((capsule, translated_key.clone()))
    }
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
                Arg::new("input_file")
                    .allow_invalid_utf8(true)
                    .help("Path of the file to be encrypted")
                    .required(true),
                Arg::new("owner_keypair")
                    .allow_invalid_utf8(true)
                    .help("Path of the keypair to encrypt the file with")
                    .required(true),
                Arg::new("output_keys")
                    .allow_invalid_utf8(true)
                    .help("Output path for the recryption keys")
                    .required(true),
                Arg::new("output_file")
                    .allow_invalid_utf8(true)
                    .help("Output path for the new encrypted file")
                    .required(true),
            ]),
        )
        .subcommand(
            App::new("recrypt")
                .about("Translates encryption key to a new pubkey")
                .args([
                    Arg::new("recryption_keys")
                        .allow_invalid_utf8(true)
                        .help("Path of the recryption keys json file")
                        .required(true),
                    Arg::new("receiver_pubkey")
                        .help("Public key of the receiver of the file")
                        .required(true),
                    Arg::new("output")
                        .allow_invalid_utf8(true)
                        .help("Output path for decryption keys")
                        .required(true),
                ]),
        )
        .subcommand(
            App::new("decrypt")
                .about("Decrypts the input file using decryption and receiver keys")
                .args([
                    Arg::new("input_file")
                        .allow_invalid_utf8(true)
                        .help("Path of the file to be decrypted")
                        .required(true),
                    Arg::new("decryption_keys")
                        .allow_invalid_utf8(true)
                        .help("Path of the decryption keys json file")
                        .required(true),
                    Arg::new("receiver_keypair")
                        .allow_invalid_utf8(true)
                        .help("Path of the keypair to decrypt the file with")
                        .required(true),
                    Arg::new("output")
                        .allow_invalid_utf8(true)
                        .help("Output path for the decrypted file")
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
            // Read the keypair file
            let keypair_path = sub_matches.value_of_os("owner_keypair").unwrap();
            let file_secret: SecretKey = parse_keypair_file(&keypair_path)?;

            // Read the input file to a buffer
            let input_path = sub_matches.value_of_os("input_file").unwrap();
            let f = File::open(input_path)?;

            let output_keys = sub_matches.value_of_os("output_keys").unwrap();
            let output_file = sub_matches.value_of_os("output_file").unwrap();
            pre_threaded::encrypt_threaded(&f, file_secret, &output_keys, &output_file)?;
            Ok(())
        }
        Some(("recrypt", sub_matches)) => {
            // Read recryption keys from file
            let recryption_keys_path = sub_matches.value_of_os("recryption_keys").unwrap();
            let recryption_keys_array = std::fs::read(recryption_keys_path).unwrap();
            let recryption_keys: RecryptionKeys = serde_json::from_slice(&recryption_keys_array)?;
            let owner_secret: SecretKey =
                SecretKey::from_bytes(recryption_keys.owner_secret).unwrap();

            // Read receiver pubkey from argunment
            let receiver_public_str = sub_matches.value_of("receiver_pubkey").unwrap();
            let receiver_public_json = format!("\"{}\"", receiver_public_str); // Turns raw string into json string
            let receiver_public: PublicKey = serde_json::from_str(&receiver_public_json)?;

            // Fragmentation/verification is not used because we aren't using proxies
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
            let output_path = sub_matches.value_of_os("output").unwrap();
            std::fs::write(
                output_path,
                serde_json::to_string(&decryption_keys).unwrap(),
            )
            .unwrap();
            Ok(())
        }
        Some(("decrypt", sub_matches)) => {
            // Read the encrypted input file to a buffer
            let input_path = sub_matches.value_of_os("input_file").unwrap();
            let f = File::open(input_path)?;
            
            // Read decryption keys file
            let decryption_keys_path = sub_matches.value_of_os("decryption_keys").unwrap();
            let decryption_keys_array = std::fs::read(decryption_keys_path).unwrap();
            let mut decryption_keys: DecryptionKeys = serde_json::from_slice(&decryption_keys_array)?;

            // Read receiver secret
            let keypair_path = sub_matches.value_of_os("receiver_keypair").unwrap();
            let receiver_secret: SecretKey = parse_keypair_file(&keypair_path)?;
            
            // Decrypt the cipher
            let output_path = sub_matches.value_of_os("output").unwrap();
            decrypt_threaded(&f, output_path, receiver_secret, &mut decryption_keys)?;
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
