use std::fs;
use serde_json::Value;
use std::thread;
use std::time::Duration;

fn main() {
    println!("🤖 Neural Guardian Live Monitor");
    println!("================================\n");
    
    loop {
        thread::sleep(Duration::from_secs(10));
        if let Ok(data) = fs::read_to_string("ai_stats.json") {
            if let Ok(stats) = serde_json::from_str::<Value>(&data) {
                print!("\x1B[2J\x1B[1;1H"); // Clear screen
                println!("Total Predictions: {}", stats["total_predictions"]);
                println!("Spam Detected: {}", stats["spam_detected"]);
                if let (Some(model_used), Some(total_predictions)) = (stats["model_used"].as_f64(), stats["total_predictions"].as_f64()) {
                    println!("Model Usage: {}%", (model_used / total_predictions * 100.0) as u32);
                }
            }
        }
    }
}
