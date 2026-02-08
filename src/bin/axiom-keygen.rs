use axiom_core::zk::circuit::ZkProofSystem;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let keys_dir = if args.len() > 1 {
        args[1].clone()
    } else {
        "zk_keys".to_string()
    };
    println!("🔐 Axiom ZK-STARK Key Generation");
    println!("================================\n");
    println!("⏳ Initializing STARK proof system (no trusted setup needed!)...");
    let start = Instant::now();
    match ZkProofSystem::setup() {
        Ok(system) => {
            let setup_time = start.elapsed();
            println!("✓ Setup completed in {:.2}s", setup_time.as_secs_f64());
            println!("\n💾 Saving keys to {}...", keys_dir);
            match system.save_keys(&keys_dir) {
                Ok(_) => {
                    println!("✓ Keys saved successfully!");
                    println!("\n📊 Key Statistics:");
                    println!("  - Proving key:    Ready");
                    println!("  - Verifying key:  Ready");
                    println!("  - Storage path:   {}", keys_dir);
                    println!("\n⚠️  IMPORTANT:");
                    println!("  - Keep proving key PRIVATE (used to create proofs)");
                    println!("  - Verifying key can be public (used by network)");
                    println!("  - Back up keys securely!");
                }
                Err(e) => {
                    eprintln!("❌ Failed to save keys: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Setup failed: {}", e);
            std::process::exit(1);
        }
    }
}
