/// OpenClaw integration - runs automatically with the node
/// Handles ceremony coordination and health monitoring in background
/// Spawns Python-based agents for security, network optimization, and monitoring
///
/// The `AgentManager` wraps each agent in a persistent `tokio::spawn` loop
/// with a mandatory 10-second back-off timer before any restart to stabilize
/// PIDs and OS resources.

use tokio::task::JoinHandle;
use std::process::{Command, Child, Stdio};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;

/// Mandatory back-off duration (seconds) before restarting any agent.
/// Prevents crash-loops from destabilizing PIDs and OS resources.
const AGENT_RESTART_BACKOFF_SECS: u64 = 10;

/// Manages the lifecycle of all OpenClaw agents with persistent
/// `tokio::spawn` loops and mandatory restart back-off timers.
struct AgentManager {
    handles: Vec<JoinHandle<()>>,
}

impl AgentManager {
    fn new() -> Self {
        Self { handles: Vec::new() }
    }

    /// Spawn a persistent agent loop. Each agent is wrapped in its own
    /// `tokio::spawn` task that monitors the process and applies a
    /// mandatory 10-second back-off before any restart.
    fn spawn_agent(
        &mut self,
        base_dir: PathBuf,
        script_name: &'static str,
        agent_name: &'static str,
        weights_path: PathBuf,
        genesis_pulse_path: PathBuf,
    ) {
        let handle = tokio::spawn(async move {
            loop {
                let child = start_agent(&base_dir, script_name, agent_name, &weights_path, &genesis_pulse_path);
                match child {
                    Some(mut proc) => {
                        // Wait for the process to exit
                        loop {
                            match proc.try_wait() {
                                Ok(None) => {
                                    // Still running — check again after a short sleep
                                    sleep(Duration::from_secs(2)).await;
                                }
                                Ok(Some(status)) => {
                                    println!("⚠️  {} exited: {} — restarting after {}s back-off",
                                        agent_name, status, AGENT_RESTART_BACKOFF_SECS);
                                    break;
                                }
                                Err(e) => {
                                    println!("⚠️  Error checking {}: {} — restarting after {}s back-off",
                                        agent_name, e, AGENT_RESTART_BACKOFF_SECS);
                                    break;
                                }
                            }
                        }
                    }
                    None => {
                        println!("⚠️  {} failed to start — retrying after {}s back-off",
                            agent_name, AGENT_RESTART_BACKOFF_SECS);
                    }
                }
                // Mandatory back-off before restart to stabilize PIDs and OS resources
                sleep(Duration::from_secs(AGENT_RESTART_BACKOFF_SECS)).await;
            }
        });
        self.handles.push(handle);
    }
}

pub async fn start_openclaw_background() -> Result<JoinHandle<()>, Box<dyn std::error::Error + Send + Sync>> {
    // Determine OpenClaw config path
    let config_path = env::var("AXIOM_OPENCLAW_CONFIG")
        .unwrap_or_else(|_| "./openclaw/bootstrap_server_config.json".to_string());
    
    // Get base directory for agents
    let base_dir = env::current_dir()?;

    // Resolve absolute paths for weights.bin and config/genesis_pulse.json
    let weights_path = base_dir.join("weights.bin").canonicalize()
        .unwrap_or_else(|_| base_dir.join("weights.bin"));
    let genesis_pulse_path = base_dir.join("config").join("genesis_pulse.json").canonicalize()
        .unwrap_or_else(|_| base_dir.join("config").join("genesis_pulse.json"));
    
    // Spawn background task that manages all OpenClaw agents
    let handle = tokio::spawn(async move {
        match run_openclaw_daemon(&config_path, &base_dir, &weights_path, &genesis_pulse_path).await {
            Ok(_) => println!("✅ OpenClaw agents terminated gracefully"),
            Err(e) => eprintln!("⚠️  OpenClaw error: {}", e),
        }
    });
    
    Ok(handle)
}

async fn run_openclaw_daemon(
    config_path: &str,
    base_dir: &std::path::Path,
    weights_path: &std::path::Path,
    genesis_pulse_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🚀 OpenClaw daemon starting...");
    println!("📁 Config: {}", config_path);
    println!("📁 Weights: {}", weights_path.display());
    println!("📁 Genesis Pulse: {}", genesis_pulse_path.display());
    
    // Check if Python is available
    let python_check = Command::new("python3")
        .arg("--version")
        .output();
    
    match python_check {
        Ok(_) => println!("✅ Python3 found - agents will be launched"),
        Err(_) => {
            println!("⚠️  Python3 not found - agents will not start");
            println!("    Install Python3 to enable: sudo apt install python3");
            return Ok(());
        }
    }
    
    let mut manager = AgentManager::new();
    let base = base_dir.to_path_buf();
    let weights = weights_path.to_path_buf();
    let genesis = genesis_pulse_path.to_path_buf();

    // Each agent is wrapped in a persistent tokio::spawn loop with
    // mandatory 10-second back-off before any restart.
    manager.spawn_agent(base.clone(), "security_guardian_agent.py", "🛡️  SECURITY GUARDIAN", weights.clone(), genesis.clone());
    manager.spawn_agent(base.clone(), "network_booster_agent.py", "🚀 NETWORK BOOSTER", weights.clone(), genesis.clone());
    manager.spawn_agent(base.clone(), "node_health_monitor.py", "🏥 HEALTH MONITOR", weights.clone(), genesis.clone());
    manager.spawn_agent(base.clone(), "ceremony_master.py", "📜 CEREMONY COORDINATOR", weights.clone(), genesis.clone());

    // Keep the daemon alive while agents run in their own tokio tasks
    loop {
        sleep(Duration::from_secs(60)).await;
    }
}

fn start_agent(
    base_dir: &std::path::Path,
    script_name: &str,
    agent_name: &str,
    weights_path: &std::path::Path,
    genesis_pulse_path: &std::path::Path,
) -> Option<Child> {
    let script_path = base_dir.join("openclaw").join(script_name);
    
    if !Path::new(&script_path).exists() {
        println!("⚠️  {} agent not found at: {}", agent_name, script_path.display());
        return None;
    }
    
    match Command::new("python3")
        .arg(script_path.to_string_lossy().to_string())
        .env("AXIOM_WEIGHTS_PATH", weights_path.to_string_lossy().to_string())
        .env("AXIOM_GENESIS_PULSE_PATH", genesis_pulse_path.to_string_lossy().to_string())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => {
            println!("✅ {} agent started (PID: {})", agent_name, child.id());
            Some(child)
        }
        Err(e) => {
            println!("❌ Failed to start {} agent: {}", agent_name, e);
            None
        }
    }
}
