/// Axiom Health Check Client
///
/// Queries the running axiom-node's built-in API to check health status.
/// The node exposes /v1/status and /v1/health/check endpoints — this
/// binary is a lightweight CLI probe suitable for systemd watchdogs,
/// Kubernetes liveness probes, and operator scripts.
///
/// Usage:
///   axiom-healthcheck                       # default: http://127.0.0.1:3030
///   axiom-healthcheck http://10.0.0.1:3030  # custom node address
///   AXIOM_API_URL=http://10.0.0.1:3030 axiom-healthcheck

use std::process;

#[tokio::main]
async fn main() {
    let default_url = "http://127.0.0.1:3030".to_string();
    let base_url = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("AXIOM_API_URL").ok())
        .unwrap_or(default_url);

    println!("🏥 Axiom Health Check");
    println!("   Node API: {}", base_url);
    println!();

    // 1. Liveness: can we reach the node at all?
    let status_url = format!("{}/v1/status", base_url);
    let health_url = format!("{}/v1/health/check", base_url);

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap_or_else(|e| {
            eprintln!("❌ Failed to create HTTP client: {}", e);
            process::exit(1);
        });

    // Check /v1/health/check (simple liveness)
    match client.get(&health_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            println!("✅ Liveness: ALIVE ({})", health_url);
        }
        Ok(resp) => {
            eprintln!("⚠️  Liveness: DEGRADED (HTTP {})", resp.status());
        }
        Err(e) => {
            eprintln!("❌ Liveness: UNREACHABLE — {}", e);
            eprintln!("   Is axiom-node running? Check with: ps aux | grep axiom-node");
            process::exit(1);
        }
    }

    // Check /v1/status (detailed metrics)
    match client.get(&status_url).send().await {
        Ok(resp) if resp.status().is_success() => {
            match resp.json::<serde_json::Value>().await {
                Ok(status) => {
                    println!("✅ Status: CONNECTED");
                    if let Some(height) = status.get("current_height") {
                        println!("   ⛓️  Chain height: {}", height);
                    }
                    if let Some(remaining) = status.get("supply_remaining_axm") {
                        println!("   💰 Supply remaining: {} AXM", remaining);
                    }
                    if let Some(pulse) = status.get("trust_pulse").and_then(|v| v.as_str()) {
                        if !pulse.is_empty() {
                            println!("   🔐 Trust pulse: {}…", &pulse[..pulse.len().min(24)]);
                        }
                    }
                    println!();
                    println!("🟢 Node is healthy");
                }
                Err(e) => {
                    eprintln!("⚠️  Status: Response parse error — {}", e);
                    process::exit(1);
                }
            }
        }
        Ok(resp) => {
            eprintln!("⚠️  Status: HTTP {}", resp.status());
            process::exit(1);
        }
        Err(e) => {
            eprintln!("❌ Status: Failed — {}", e);
            process::exit(1);
        }
    }
}
