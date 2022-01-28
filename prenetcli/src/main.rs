use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

use clap::{arg, App, AppSettings};
use umbral_pre::*;

fn decrypt(
    buyer_secret: &SecretKey,
    file_pub: &PublicKey,
    capsule: &Capsule,
    verified_cfrag: VerifiedCapsuleFrag,
    ciphertext: impl AsRef<[u8]>,
) {
    let plaintext_buyer = decrypt_reencrypted(
        &buyer_secret,
        &file_pub,
        &capsule,
        [verified_cfrag],
        &ciphertext,
    )
    .unwrap();

    let plain_vec = plaintext_buyer.to_vec();
    println!("{}", String::from_utf8(plain_vec).unwrap());
    return;
}

fn main() -> std::io::Result<()> {
    let matches = App::new("prenet")
        .about("Cli for pre-network")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .subcommand(
            App::new("encrypt")
                .about("Encrypts file")
                .arg(arg!(<PATH> ... "Stuff to encrypt").allow_invalid_utf8(true)),
        )
        // .subcommand(
        //     App::new("translate")
        //         .about("Translates encryption key to a new pubkey")
        //         .arg(arg!(<PATH> ... "Stuff to encrypt").allow_invalid_utf8(true)),
        // )
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
                
                // Create a new random secret for this file
                let file_secret = SecretKey::random();
                // println!("File Secret: {}")
                let file_pub = file_secret.public_key();

                // Use that key to encrypt it
                let (capsule, ciphertext) = encrypt(&file_pub, plaintext.as_bytes()).unwrap();
                
                // let encoded_cap: Vec<u8> = bincode::serialize(&capsule).unwrap();
                // let encoded_cipher: Vec<u8> = bincode::serialize(&ciphertext).unwrap();
                // std::fs::write("capsule.bin", encoded_cap).unwrap();
                // std::fs::write("cipher.bin", encoded_cipher).unwrap();


                let buyer_secret = SecretKey::random();
                let buyer_public = buyer_secret.public_key();

                // Code the server runs
                // Fragmentation/verification is not used because we aren't using proxies
                let translation_key =
                    generate_kfrags(&file_secret, &buyer_public, &Signer::new(SecretKey::random()), 1, 1, false, false)[0]
                    .clone();
                let translated_key = reencrypt(&capsule, translation_key);

                // Code the buyer runs
                decrypt(
                    &buyer_secret,
                    &file_pub,
                    &capsule,
                    translated_key,
                    &ciphertext,
                );
            }

            Ok(())
        }

        _ => unreachable!(),
    }
}
