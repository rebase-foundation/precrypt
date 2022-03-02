import * as umbral from "@nucypher/umbral-pre";
interface RecryptionKeys {
    owner_secret: Uint8Array;
    capsules: Uint8Array[];
    chunk_size: number;
}
export declare function precrypt_file(input_path: string, file_key: umbral.SecretKey, output_path: string, threads: number, memory_size: number): Promise<void>;
interface DecryptionKeys {
    owner_pubkey: Uint8Array;
    capsules: Uint8Array[];
    translated_keys: Uint8Array[];
    chunk_size: number;
}
export declare function recrypt_keys(recryption_keys: RecryptionKeys, receiver_public: umbral.PublicKey): DecryptionKeys;
export declare function decrypt_file(input_path: string, output_path: string, receiver_key: umbral.SecretKey, decryption_keys: DecryptionKeys, threads: number): void;
export {};
