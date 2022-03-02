"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.decrypt_file = exports.recrypt_keys = exports.precrypt_file = void 0;
const tslib_1 = require("tslib");
const fs = (0, tslib_1.__importStar)(require("fs"));
const umbral = (0, tslib_1.__importStar)(require("@nucypher/umbral-pre"));
const CHUNK_SIZE = 100000000; // 100MB
async function precrypt_file(input_path, file_key, output_path, threads, memory_size) {
    if (memory_size % threads != 0) {
        throw new Error("'memory_size' must be evenly divisible by 'threads'");
    }
    // var buffer = Buffer.alloc(CHUNK_SIZE);
    // fs.open(input_path, 'r+', function (err, fd) {
    //    if (err) {
    //       return console.error(err);
    //    }
    //    var i = 0;
    //    var reading = true;
    //    console.log("Reading");
    //    while (i < (CHUNK_SIZE * 3)) {
    //       console.log("Read");
    //       fs.read
    //       i += 1;
    //    }
    //    // Close the opened file.
    //    fs.close(fd, function (err) {
    //       if (err) {
    //          console.log(err);
    //       }
    //       console.log("File closed successfully");
    //    });
    // });
    var i = 0;
    let stream = fs.createReadStream(input_path, { highWaterMark: CHUNK_SIZE, start: CHUNK_SIZE * 0 });
    for await (const buffer of stream) {
        console.log(i);
        const encrypt = umbral.encrypt;
        encrypt(file_key.publicKey(), buffer);
        i += 1;
    }
    // return {
    //    owner_secret: file_key.toSecretBytes(),
    //    capsules: capsules,
    //    chunk_size: CHUNK_SIZE
    // }
}
exports.precrypt_file = precrypt_file;
function recrypt_keys(recryption_keys, receiver_public) {
    const owner_secret = umbral.SecretKey.fromBytes(recryption_keys.owner_secret);
    const translation_key = umbral.generateKFrags(owner_secret, receiver_public, new umbral.Signer(umbral.SecretKey.random()), 1, 1, false, false);
    console.log(translation_key[0]);
    const capsule = umbral.Capsule.fromBytes(recryption_keys.capsules[0]);
    let translated_key = umbral.reencrypt(capsule, translation_key[0]);
    return {
        owner_pubkey: owner_secret.publicKey().toBytes(),
        capsules: [capsule.toBytes()],
        translated_keys: [translated_key.toBytes()],
        chunk_size: recryption_keys.chunk_size
    };
}
exports.recrypt_keys = recrypt_keys;
function decrypt_file(input_path, output_path, receiver_key, decryption_keys, threads) {
    const owner_pubkey = umbral.PublicKey.fromBytes(decryption_keys.owner_pubkey);
    let ciphertext_bytes = fs.readFileSync(input_path);
    const capsule = umbral.Capsule.fromBytes(decryption_keys.capsules[0]);
    console.log(capsule);
    let plaintext_bytes = capsule
        .withCFrag(umbral.VerifiedCapsuleFrag.fromVerifiedBytes(decryption_keys.translated_keys[0]))
        .decryptReencrypted(receiver_key, owner_pubkey, ciphertext_bytes);
    fs.writeFileSync(output_path, plaintext_bytes);
}
exports.decrypt_file = decrypt_file;
// NOW I HAVE TO MAKE THIS WORK FOR LARGE FILES
const file_key = umbral.SecretKey.random();
const input_path = "/Users/jacob/Downloads/heic1502a.tiff";
const output_path = "./cipher.txt";
// const recryption_keys = 
(async () => {
    try {
        await precrypt_file(input_path, file_key, output_path, 1, 1);
    }
    catch (e) {
        console.log(e);
    }
})();
// const receiver_key = umbral.SecretKey.random();
// const decryption_keys = recrypt_keys(recryption_keys, receiver_key.publicKey());
// decrypt_file(output_path, "./decrypted.jpeg", receiver_key, decryption_keys, 1);
