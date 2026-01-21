use ark_groth16::{Groth16, prepare_verifying_key, ProvingKey};
use ark_snark::CircuitSpecificSetupSNARK;
use ark_bls12_381::{Bls12_381, Fr};
use ark_serialize::CanonicalSerialize;
use rand::thread_rng;
use std::fs;
use std::path::Path;
use qubit_core::zk::circuit::QubitTransactionCircuit;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Starting Qubit Protocol ZK-SNARK Trusted Setup");
    println!("=================================================");

    // Create keys directory if it doesn't exist
    fs::create_dir_all("keys")?;

    // Generate random parameters for the circuit
    println!("🎲 Generating circuit parameters...");
    let mut rng = thread_rng();

    // Create a dummy circuit instance for parameter generation
    // In a real trusted setup, this would be done with contributions from multiple parties
    let circuit = QubitTransactionCircuit {
        secret_key: Some(Fr::from(12345u64)), // Dummy values for setup
        current_balance: Some(Fr::from(1000000u64)),
        nonce: None,
        commitment: None,
        transfer_amount: Some(Fr::from(500000u64)),
        fee: Some(Fr::from(1000u64)),
        new_balance_commitment: None,
    };

    println!("⚙️  Generating proving key (this may take a while)...");
    let (proving_key, verification_key) = Groth16::<Bls12_381>::setup(circuit, &mut rng)?;


    // Serialize and save the proving key
    println!("💾 Saving proving key...");
    let pk_path = Path::new("keys/proving_key.bin");
    if pk_path.exists() {
        return Err("Refusing to overwrite existing proving key in production".into());
    }
    let mut pk_file = fs::OpenOptions::new().write(true).create_new(true).open(pk_path)?;
    let proving_key: ProvingKey<Bls12_381> = proving_key;
    proving_key.serialize_compressed(&mut pk_file)?;
    // Enforce secure permissions
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(pk_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(pk_path, perms)?;
    }

    // Serialize and save the verification key as JSON for easier handling
    println!("💾 Saving verification key...");
    let vk_path = Path::new("keys/verification_key.json");
    if vk_path.exists() {
        return Err("Refusing to overwrite existing verification key in production".into());
    }

    // Convert verification key to a serializable format
    let prepared_vk = prepare_verifying_key(&verification_key);
    let mut vk_bytes = Vec::new();
    prepared_vk.serialize_compressed(&mut vk_bytes)?;

    // Save as hex-encoded JSON for easy distribution
    let vk_json = serde_json::json!({
        "protocol": "groth16",
        "curve": "bls12-381",
        "circuit": "QubitTransactionCircuit",
        "verification_key_hex": hex::encode(vk_bytes),
        "metadata": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "ceremony_version": "1.0.0",
            "constraints": "balance_verification"
        }
    });

    fs::write(vk_path, serde_json::to_string_pretty(&vk_json)?)?;
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(vk_path)?.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(vk_path, perms)?;
    }

    // Get file sizes for logging
    let pk_size = fs::metadata(pk_path)?.len();
    let vk_size = fs::metadata(vk_path)?.len();

    println!("✅ Setup complete!");
    println!("📊 File sizes:");
    println!("   - proving_key.bin: {} bytes ({:.2} MB)", pk_size, pk_size as f64 / 1_000_000.0);
    println!("   - verification_key.json: {} bytes", vk_size);

    // Calculate and display hashes for verification
    let pk_hash = sha256_file(pk_path)?;
    let vk_hash = sha256_file(vk_path)?;

    println!("🔒 Key hashes (SHA256):");
    println!("   - proving_key.bin: {}", pk_hash);
    println!("   - verification_key.json: {}", vk_hash);

    println!("\n🚨 SECURITY NOTICE:");
    println!("   - proving_key.bin contains sensitive cryptographic material");
    println!("   - Upload to decentralized storage (IPFS/Arweave)");
    println!("   - NEVER commit to version control");
    println!("   - Destroy toxic waste after distribution");

    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    use sha2::{Sha256, Digest};
    use std::io::Read;

    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}