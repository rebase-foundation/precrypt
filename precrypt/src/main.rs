use clap::Arg;
use clap::{App, AppSettings};
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs::File;
use std::io::BufReader;
use umbral_pre::*;

mod lib;
pub use crate::lib::*;

#[derive(Serialize, Deserialize, Clone)]
struct Keypair {
    public_key: PublicKey,
    secret_key: Vec<u8>,
}

fn parse_keypair_file(path: &OsStr) -> std::io::Result<SecretKey> {
    let file = File::open(path)?;
    let keypair_json: Keypair = serde_json::from_reader(BufReader::new(file))?;
    return Ok(SecretKey::from_bytes(keypair_json.secret_key).unwrap());
}

fn main() -> std::io::Result<()> {
    let matches = App::new("precrypt")
        .about("Cli for pre-network")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("encrypt")
                .about("Encrypts file with proxy based re-encryption")
                .args([
                    Arg::new("input_file")
                        .help("Path of the file to be encrypted")
                        .allow_invalid_utf8(true)
                        .takes_value(true)
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
                    Arg::new("threads")
                        .short('t')
                        .long("threads")
                        .validator(|s| s.parse::<usize>())
                        .default_value("10")
                        .help("Number of threads to use for parallel encryption")
                        .required(false)
                        .takes_value(true),
                    Arg::new("memory_size")
                        .short('m')
                        .long("memory_size")
                        .validator(|s| s.parse::<usize>())
                        .default_value("50000000")
                        .help("Maximum number of bytes to be stored in memory at once")
                        .required(false)
                        .takes_value(true),
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
                    Arg::new("threads")
                        .short('t')
                        .long("threads")
                        .validator(|s| s.parse::<usize>())
                        .default_value("10")
                        .help("Number of threads to use for parallel encryption")
                        .required(false)
                        .takes_value(true)
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

            // Read the input file path
            let input_path = sub_matches.value_of_os("input_file").unwrap();
            let output_keys = sub_matches.value_of_os("output_keys").unwrap();
            let output_file = sub_matches.value_of_os("output_file").unwrap();

            let threads: usize = sub_matches.value_of_t("threads").unwrap();
            let memory_size: usize = sub_matches.value_of_t("memory_size").unwrap();

            lib::precrypt(
                input_path,
                file_secret,
                &output_keys,
                &output_file,
                threads,
                memory_size,
            )?;
            Ok(())
        }
        Some(("recrypt", sub_matches)) => {
            // Read recryption keys from file
            let recryption_keys_path = sub_matches.value_of_os("recryption_keys").unwrap();
            let recryption_keys_array = std::fs::read(recryption_keys_path).unwrap();
            let recryption_keys: RecryptionKeys = serde_json::from_slice(&recryption_keys_array)?;

            // Read receiver pubkey from argument
            let receiver_public_str = sub_matches.value_of("receiver_pubkey").unwrap();
            let receiver_public_json = format!("\"{}\"", receiver_public_str); // Turns raw string into json string
            let receiver_public: PublicKey = serde_json::from_str(&receiver_public_json)?;

            let output_path = sub_matches.value_of_os("output").unwrap();
            recrypt(recryption_keys, receiver_public, output_path)?;
            Ok(())
        }
        Some(("decrypt", sub_matches)) => {
            // Read the encrypted input file path
            let input_path = sub_matches.value_of_os("input_file").unwrap();
            // Read decryption keys file
            let decryption_keys_path = sub_matches.value_of_os("decryption_keys").unwrap();
            let decryption_keys_array = std::fs::read(decryption_keys_path).unwrap();
            let mut decryption_keys: DecryptionKeys =
                serde_json::from_slice(&decryption_keys_array)?;

            // Read receiver secret
            let keypair_path = sub_matches.value_of_os("receiver_keypair").unwrap();
            let receiver_secret: SecretKey = parse_keypair_file(&keypair_path)?;
            // Decrypt the cipher
            let output_path = sub_matches.value_of_os("output").unwrap();

            let threads: usize = sub_matches.value_of_t("threads").unwrap();

            decrypt(
                input_path,
                output_path,
                receiver_secret,
                &mut decryption_keys,
                threads
            )?;
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

            // serde_json::from_str(s: &'a str)
            Ok(())
        }
        _ => unreachable!(),
    }
}
