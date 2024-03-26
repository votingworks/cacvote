use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::sign::Verifier;
use std::env;
use std::fs;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();

    // Check if correct number of arguments are provided
    if args.len() != 4 {
        println!(
            "Usage: {} <public_key_file> <payload_file> <signature_file>",
            args[0]
        );
        return;
    }

    // Extract arguments
    let public_key_path = &args[1];
    let payload_path = &args[2];
    let signature_path = &args[3];

    // Read public key (X.509 certificate)
    let public_key_content = match fs::read(public_key_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Error: Unable to read public key file");
            return;
        }
    };
    let public_key = PKey::public_key_from_pem(&public_key_content).unwrap();

    // Read payload
    let payload_content = match fs::read(payload_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Error: Unable to read payload file");
            return;
        }
    };

    // Read signature
    let signature_content = match fs::read(signature_path) {
        Ok(content) => content,
        Err(_) => {
            println!("Error: Unable to read signature file");
            return;
        }
    };

    // Verify signature
    let mut verifier = match Verifier::new(MessageDigest::sha256(), &public_key) {
        Ok(verifier) => verifier,
        Err(_) => {
            println!("Error: Unable to create verifier");
            return;
        }
    };
    match verifier.verify_oneshot(&signature_content, &payload_content) {
        Err(_) | Ok(false) => {
            println!("Signature verification failed");
        }
        Ok(true) => {
            println!("Signature verified successfully");
        }
    }
}
