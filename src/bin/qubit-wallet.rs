use std::fs;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: qubit-wallet [export|show|send|balance]");
        println!("  export     - Show wallet address in hex format");
        println!("  show       - Show full wallet details");
        println!("  balance    - Show current balance");
        println!("  send <to> <amount> <fee> - Send QBT to address");
        return;
    }

    let command = &args[1];

    // Load wallet
    let wallet_data = match fs::read("wallet.dat") {
        Ok(data) => data,
        Err(_) => {
            eprintln!("❌ Error: wallet.dat not found. Run the qubit node first to generate a wallet.");
            std::process::exit(1);
        }
    };

    // Deserialize wallet
    let wallet: qubit_core::wallet::Wallet = match bincode::deserialize(&wallet_data) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("❌ Error deserializing wallet: {}", e);
            std::process::exit(1);
        }
    };

    match command.as_str() {
        "export" => {
            println!("{}", hex::encode(wallet.address));
        }
        "show" => {
            println!("💳 Qubit Wallet Details");
            println!("=======================");
            println!("Address (hex): {}", hex::encode(wallet.address));
            println!("Address length: {} bytes", wallet.address.len());
            println!("⚠️  KEEP wallet.dat SAFE - it contains your secret key!");
        }
        "balance" => {
            // Load chain to get balance
            let chain_data = match fs::read("qubit_chain.dat") {
                Ok(data) => data,
                Err(_) => {
                    println!("No blockchain data found. Balance: 0 QBT");
                    return;
                }
            };

            let blocks: Vec<qubit_core::block::Block> = match bincode::deserialize(&chain_data) {
                Ok(blocks) => blocks,
                Err(e) => {
                    eprintln!("❌ Error deserializing chain: {}", e);
                    std::process::exit(1);
                }
            };

            // Calculate balance
            let mut balance = 0u64;
            for block in &blocks {
                // Add mining rewards
                if block.miner == wallet.address {
                    let halvings = block.slot / 2_100_000;
                    let reward = 50_000_000_000u64 >> halvings;
                    balance += reward;
                }

                // Process transactions
                for tx in &block.transactions {
                    if tx.to == wallet.address {
                        balance += tx.amount;
                    }
                    if tx.from == wallet.address {
                        balance -= tx.amount + tx.fee;
                    }
                }
            }

            let qbt_balance = balance as f64 / 100_000_000.0;
            println!("💰 Balance: {:.8} QBT", qbt_balance);
        }
        "send" => {
            if args.len() < 5 {
                eprintln!("Usage: qubit-wallet send <to_address_hex> <amount_qbt> <fee_qbt>");
                std::process::exit(1);
            }

            let to_hex = &args[2];
            let amount_qbt: f64 = match args[3].parse() {
                Ok(a) => a,
                Err(_) => {
                    eprintln!("❌ Invalid amount");
                    std::process::exit(1);
                }
            };
            let fee_qbt: f64 = match args[4].parse() {
                Ok(f) => f,
                Err(_) => {
                    eprintln!("❌ Invalid fee");
                    std::process::exit(1);
                }
            };

            // Convert to smallest units
            let amount = (amount_qbt * 100_000_000.0) as u64;
            let fee = (fee_qbt * 100_000_000.0) as u64;

            // Decode recipient address
            let to_address = match hex::decode(to_hex) {
                Ok(bytes) if bytes.len() == 32 => {
                    let mut addr = [0u8; 32];
                    addr.copy_from_slice(&bytes);
                    addr
                }
                _ => {
                    eprintln!("❌ Invalid recipient address");
                    std::process::exit(1);
                }
            };

            // Get current balance and nonce (simplified)
            let current_balance = 1_000_000_000_000; // Placeholder - should load from chain
            let nonce = 0; // Placeholder - should track per address

            // Create transaction
            match wallet.create_transaction(to_address, amount, fee, nonce, current_balance) {
                Ok(tx) => {
                    // Save transaction to file for broadcasting
                    let tx_data = match bincode::serialize(&tx) {
                        Ok(data) => data,
                        Err(e) => {
                            eprintln!("❌ Error serializing transaction: {}", e);
                            std::process::exit(1);
                        }
                    };

                    match fs::write("pending_tx.dat", tx_data) {
                        Ok(_) => {
                            println!("✅ Transaction created and saved to pending_tx.dat");
                            println!("📤 Run the qubit node to broadcast this transaction");
                            println!("From: {}", hex::encode(tx.from));
                            println!("To: {}", hex::encode(tx.to));
                            println!("Amount: {:.8} QBT", amount as f64 / 100_000_000.0);
                            println!("Fee: {:.8} QBT", fee as f64 / 100_000_000.0);
                        }
                        Err(e) => {
                            eprintln!("❌ Error saving transaction: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error creating transaction: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("❌ Unknown command: {}", command);
            eprintln!("Use 'export', 'show', 'balance', or 'send'");
            std::process::exit(1);
        }
    }
}
