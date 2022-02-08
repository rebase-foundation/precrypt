import fs from 'fs';
import * as umbral from "@nucypher/umbral-pre";

let enc = new TextEncoder();
let dec = new TextDecoder("utf-8");

// READ INPUTS
console.log("Inputs:")
let plaintext = fs.readFileSync("../plaintext.txt");
let plaintext_bytes = new Uint8Array(plaintext);
console.log(`   Plaintext: [${plaintext_bytes}]`);
let secret_raw = fs.readFileSync("../secret.json");
let secret_bytes = JSON.parse(secret_raw.toString());
console.log(`   Secret: ${secret_bytes}`);

// ENCRYPT
let secret = umbral.SecretKey.fromBytes(secret_bytes);
let result = umbral.encrypt(secret.publicKey(), plaintext_bytes);
let capsule = result.capsule;
let cipher = result.ciphertext;

// PRINT OUTPUTS
console.log("\nOutputs:")
console.log(`   Capsule: [${capsule.toBytes()}]`)
console.log(`   Cipher: [${cipher}]`)

// WRITE CAPSULE
fs.writeFileSync("../js_capsule.json", `[${capsule.toBytes()}]`);

// TEST JS CAPSULE
console.log();
let js_capsule_raw = fs.readFileSync("../js_capsule.json");
let js_capsule_bytes = JSON.parse(js_capsule_raw.toString());
console.log(`JS capsule length: ${js_capsule_bytes.length}`);
try {
   let _ = umbral.Capsule.fromBytes(js_capsule_bytes);
   console.log("Successfully parsed JS capsule")
} catch {
   console.log("FAILED parsing JS capsule")
}

// TEST RUST CAPSULE
let rust_capsule_raw = fs.readFileSync("../rust_capsule.json");
let rust_capsule_bytes = JSON.parse(rust_capsule_raw.toString());
console.log(`Rust capsule length: ${rust_capsule_bytes.length}`);
try {
   let _ = umbral.Capsule.fromBytes(rust_capsule_bytes);
   console.log("Successfully parsed Rust capsule")
} catch {
   console.log("FAILED parsing Rust capsule")
}