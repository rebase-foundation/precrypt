# Precrypt: Rust CLI for proxy-based re-encryption.

This is a CLI wrapper for the [umbral-pre](https://github.com/nucypher/rust-umbral/blob/master/umbral-pre/README.md) proxy-based re-encryption library. It adds parallel processing for encryption and decryption that makes it possible to handle large files. 

## Usage

### 1) Encrypt your file

Generate a keypair that will be used to encrypt the file. **Keep this key private**, anyone who has access to it can decrypt your file.

```
precrypt keygen key.json
```

Encrypt your target file with you're keypair.

``` 
precrypt encrypt secret.txt key.json recrypt.json out.txt
```

> `encrypt` uses 10 threads by default, you can adjust this with the `-t` argument.

**Note:** We did not need a recipients public key when encrypting the file. This is the magic of proxy re-encryption, you can *re-encrypt* the file to a new public key at any point using a re-encryption key! This saves compute resources because you only need to encrypt the file once.

### 2) Recrypt your file to a public key

To give someone access to a file you will need their public key (they can generate one using the `keygen` command).

```
precrypt recrypt recrypt.json <pubkey> decrypt.json
```

This will create a decryption key that they can combine with their secret key to decrypt the file.

### 3) Decrypt the file

The recipient can now decrypt the file using their private key and the decryption key.

```
precrypt decrypt out.txt decrypt.json recipient_key.json decrypted_secret.txt
```