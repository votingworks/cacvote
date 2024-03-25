use openssl::x509::X509;
use std::env;
use std::fs;

fn main() {
    // Get command-line arguments
    let args: Vec<String> = env::args().collect();

    // Check if enough arguments are provided
    if args.len() != 3 {
        eprintln!("Usage: {} <path_to_cert1> <path_to_cert2>", args[0]);
        return;
    }

    // Read certificate files
    let cert1_bytes = fs::read(&args[1]).expect("Unable to read file");
    let cert2_bytes = fs::read(&args[2]).expect("Unable to read file");

    // Parse certificates
    let cert1 = X509::from_pem(&cert1_bytes).expect("Failed to parse certificate");
    let cert2 = X509::from_pem(&cert2_bytes).expect("Failed to parse certificate");

    // Verify if cert2 has been signed by cert1
    let cert1_public_key = cert1.public_key().unwrap();
    if cert2.verify(&cert1_public_key).unwrap() {
        println!("OK: {} has been signed by {}", args[2], args[1]);
    } else {
        println!("FAIL: {} has not been signed by {}", args[2], args[1]);
    }
}
