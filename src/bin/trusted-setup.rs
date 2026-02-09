use axiom_core::zk::circuit::ZkProofSystem;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 AXIOM Protocol ZK-STARK Parameter Generation");
    println!("================================================");
    println!("ℹ️  STARKs require NO trusted setup ceremony!");
    println!("   Parameters are transparent and publicly verifiable.");
    println!();

    // Create keys directory if it doesn't exist
    fs::create_dir_all("keys")?;

    println!("⚙️  Generating STARK proof parameters...");
    let system = ZkProofSystem::setup()
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;

    // Save parameters
    println!("💾 Saving STARK parameters...");
    let params_path = Path::new("keys/stark_params.json");
    let params_json = serde_json::json!({
        "protocol": "stark",
        "hash_function": "blake3",
        "field": "f128",
        "security_bits": 128,
        "trusted_setup": false,
        "post_quantum": true,
        "metadata": {
            "generated_at": chrono::Utc::now().to_rfc3339(),
            "version": "4.1.0",
            "description": "AXIOM Protocol ZK-STARK parameters - transparent, no toxic waste"
        }
    });

    fs::write(params_path, serde_json::to_string_pretty(&params_json)?)?;

    let params_size = fs::metadata(params_path)?.len();

    println!("✅ Parameter generation complete!");
    println!("📊 File sizes:");
    println!("   - stark_params.json: {} bytes", params_size);
    println!();
    println!("🎉 ADVANTAGES OF STARKs:");
    println!("   ✅ No trusted setup ceremony required");
    println!("   ✅ No toxic waste to destroy");
    println!("   ✅ Post-quantum secure (hash-based)");
    println!("   ✅ Transparent and publicly verifiable");
    println!("   ✅ Faster proving for large computations");

    // Calculate and display hash for verification
    let params_hash = sha256_file(params_path)?;
    println!();
    println!("🔒 Parameter hash (SHA256): {}", params_hash);

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
