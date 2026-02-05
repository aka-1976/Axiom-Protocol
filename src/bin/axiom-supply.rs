use std::fs;

fn main() {
    // Load chain from storage
    let mut chain_path = std::path::PathBuf::from("./axiom_chain.dat");
    
    // Try alternate paths in case called from different dir
    if !chain_path.exists() {
        chain_path = std::path::PathBuf::from("axiom_chain.dat");
    }
    if !chain_path.exists() {
        chain_path = std::path::PathBuf::from("../axiom_chain.dat");
    }

    let blocks = match fs::read(&chain_path) {
        Ok(data) => {
            match bincode::deserialize::<Vec<axiom_core::block::Block>>(&data) {
                Ok(blocks) => blocks,
                Err(_) => {
                    eprintln!("❌ Error: Could not deserialize chain blocks.");
                    eprintln!("   Make sure axiom-supply is run from the same directory as axiom_chain.dat");
                    std::process::exit(1);
                }
            }
        }
        Err(_) => {
            eprintln!("❌ Error: axiom_chain.dat not found.");
            eprintln!("   Run the axiom node first to generate a chain.");
            eprintln!("   Path searched: {:?}", chain_path);
            std::process::exit(1);
        }
    };

    // Calculate total mined
    let mut total_mined: u64 = 0;
    for (i, _block) in blocks.iter().enumerate() {
        let halvings = i as u64 / 1_240_000;
        let reward = 50_000_000_000u64 >> halvings;
        total_mined = total_mined.saturating_add(reward);
    }

    let max_supply: u64 = 124_000_000_000_000_000;
    let remaining = max_supply.saturating_sub(total_mined);
    let percent = (total_mined as f64 / max_supply as f64) * 100.0;

    // Format to AXM (8 decimals)
    let decimals = 8u32;
    let mined_axm = total_mined as f64 / (10_u64.pow(decimals) as f64);
    let remaining_axm = remaining as f64 / (10_u64.pow(decimals) as f64);
    let max_axm = max_supply as f64 / (10_u64.pow(decimals) as f64);

    println!("\n💰 AXIOM SUPPLY STATUS");
    println!("======================");
    println!("Total Mined:        {:.8} AXM", mined_axm);
    println!("Total Remaining:    {:.8} AXM", remaining_axm);
    println!("Max Supply:         {:.8} AXM", max_axm);
    println!("Percentage Mined:   {:.2}%", percent);
    println!("Current Height:     {}", blocks.len());
    println!("======================\n");
}
