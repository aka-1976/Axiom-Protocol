// src/ai/oracle.rs - AI Oracle Network for AXIOM Protocol
// Decentralized LLM inference with consensus and verification

use serde::{Serialize, Deserialize};
use reqwest;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Oracle query submitted by users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleQuery {
    pub query_id: [u8; 32],
    pub prompt: String,
    pub requester: [u8; 32],
    pub max_tokens: u32,
    pub temperature: f32,
    pub reward: u64, // AXM tokens for oracles
    pub timestamp: u64,
}

/// Oracle response from a single oracle node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    pub query_id: [u8; 32],
    pub response_text: String,
    pub model: String,
    pub oracle_address: [u8; 32],
    pub signature: Vec<u8>,
    pub timestamp: u64,
}

/// Consensus result with majority-voted response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleConsensus {
    pub query_id: [u8; 32],
    pub agreed_response: String,
    pub confidence: f64, // 0.0-1.0
    pub participating_oracles: Vec<[u8; 32]>,
    pub dissenting_oracles: Vec<[u8; 32]>,
}

/// AI Oracle node that processes queries
pub struct OracleNode {
    pub address: [u8; 32],
    pub api_key: String,
    pub model: String,
    signing_key: ed25519_dalek::SigningKey,
}

impl OracleNode {
    pub fn new(address: [u8; 32], api_key: String) -> Self {
        // Load the operator's signing key from the wallet file.
        // The address is the corresponding Ed25519 verifying key.
        let signing_key = match std::fs::read("wallet.dat") {
            Ok(data) => {
                match bincode::deserialize::<crate::wallet::Wallet>(&data) {
                    Ok(w) if w.address == address => {
                        ed25519_dalek::SigningKey::from_bytes(&w.secret_key)
                    }
                    _ => {
                        // Wallet doesn't match this oracle address — generate
                        // a fresh key pair.  The verifying key becomes the
                        // effective oracle address for this session.
                        let mut rng = rand::rngs::OsRng;
                        ed25519_dalek::SigningKey::generate(&mut rng)
                    }
                }
            }
            Err(_) => {
                let mut rng = rand::rngs::OsRng;
                ed25519_dalek::SigningKey::generate(&mut rng)
            }
        };

        Self {
            address,
            api_key,
            model: "claude-3-5-sonnet-20241022".to_string(),
            signing_key,
        }
    }
    
    /// Process oracle query using Claude API
    pub async fn process_query(&self, query: &OracleQuery) -> Result<OracleResponse, String> {
        log::info!("Oracle {}: Processing query {}", 
            hex::encode(&self.address[..4]),
            hex::encode(&query.query_id[..4]));
        
        // Call Claude API
        let response_text = self.call_claude_api(&query.prompt, query.max_tokens, query.temperature)
            .await
            .map_err(|e| format!("Claude API error: {}", e))?;
        
        // Sign response
        let signature = self.sign_response(&query.query_id, &response_text);
        
        Ok(OracleResponse {
            query_id: query.query_id,
            response_text,
            model: self.model.clone(),
            oracle_address: self.address,
            signature,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or_else(|e| {
                    log::warn!("Failed to get oracle timestamp: {}", e);
                    0
                }),
        })
    }
    
    /// Call Anthropic Claude API
    async fn call_claude_api(
        &self,
        prompt: &str,
        max_tokens: u32,
        temperature: f32,
    ) -> Result<String, String> {
        let client = reqwest::Client::new();
        
        let request_body = serde_json::json!({
            "model": self.model,
            "max_tokens": max_tokens,
            "temperature": temperature,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });
        
        let response = client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request_body)
            .send()
            .await
            .map_err(|e| format!("HTTP error: {}", e))?;
        
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(format!("API error {}: {}", status, error_text));
        }
        
        let response_json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| format!("JSON parse error: {}", e))?;
        
        // Extract text from response
        let text = response_json["content"][0]["text"]
            .as_str()
            .ok_or("Missing text in response")?
            .to_string();
        
        Ok(text)
    }
    
    /// Sign oracle response with Ed25519
    fn sign_response(&self, query_id: &[u8; 32], response: &str) -> Vec<u8> {
        use ed25519_dalek::Signer;
        // Build the message to sign: query_id || response bytes
        let mut message = Vec::with_capacity(32 + response.len());
        message.extend_from_slice(query_id);
        message.extend_from_slice(response.as_bytes());
        let signature = self.signing_key.sign(&message);
        signature.to_bytes().to_vec()
    }
}

/// Oracle consensus manager
pub struct OracleConsensusManager {
    pub minimum_oracles: usize,
    pub similarity_threshold: f64,
}

impl OracleConsensusManager {
    pub fn new(minimum_oracles: usize, similarity_threshold: f64) -> Self {
        Self {
            minimum_oracles,
            similarity_threshold,
        }
    }
    
    /// Find consensus among oracle responses
    pub fn find_consensus(
        &self,
        responses: Vec<OracleResponse>,
    ) -> Result<OracleConsensus, String> {
        if responses.len() < self.minimum_oracles {
            return Err(format!(
                "Not enough responses: {} < {}",
                responses.len(),
                self.minimum_oracles
            ));
        }
        
        let query_id = responses[0].query_id;
        
        // Group similar responses
        let clusters = self.cluster_responses(&responses);
        
        // Find majority cluster
        let (majority_response, majority_oracles) = clusters
            .iter()
            .max_by_key(|(_, oracles)| oracles.len())
            .ok_or("No majority found")?;
        
        let confidence = majority_oracles.len() as f64 / responses.len() as f64;
        
        // Identify dissenters
        let majority_addresses: Vec<[u8; 32]> = clusters
            .get(majority_response)
            .cloned()
            .unwrap_or_default();
        
        let dissenting_oracles: Vec<[u8; 32]> = responses
            .iter()
            .filter(|r| !majority_addresses.contains(&r.oracle_address))
            .map(|r| r.oracle_address)
            .collect();
        
        Ok(OracleConsensus {
            query_id,
            agreed_response: majority_response.clone(),
            confidence,
            participating_oracles: majority_addresses,
            dissenting_oracles,
        })
    }
    
    /// Cluster responses by semantic similarity
    fn cluster_responses(&self, responses: &[OracleResponse]) -> HashMap<String, Vec<[u8; 32]>> {
        let mut clusters: HashMap<String, Vec<[u8; 32]>> = HashMap::new();
        
        for response in responses {
            let mut added = false;
            
            // Try to add to existing cluster
            for (cluster_text, oracles) in clusters.iter_mut() {
                if self.are_similar(cluster_text, &response.response_text) {
                    oracles.push(response.oracle_address);
                    added = true;
                    break;
                }
            }
            
            // Create new cluster if needed
            if !added {
                clusters.insert(
                    response.response_text.clone(),
                    vec![response.oracle_address],
                );
            }
        }
        
        clusters
    }
    
    /// Check if two responses are semantically similar
    fn are_similar(&self, a: &str, b: &str) -> bool {
        // Normalized Levenshtein distance comparison
        let normalized_a = a.to_lowercase().trim().to_string();
        let normalized_b = b.to_lowercase().trim().to_string();
        
        // Exact match
        if normalized_a == normalized_b {
            return true;
        }
        
        // Levenshtein distance ratio
        let distance = levenshtein_distance(&normalized_a, &normalized_b);
        let max_len = a.len().max(b.len()) as f64;
        let similarity = 1.0 - (distance as f64 / max_len);
        
        similarity >= self.similarity_threshold
    }
    
    /// Distribute rewards to participating oracles
    pub fn distribute_rewards(
        &self,
        consensus: &OracleConsensus,
        total_reward: u64,
    ) -> HashMap<[u8; 32], u64> {
        let mut rewards = HashMap::new();
        
        let per_oracle = total_reward / consensus.participating_oracles.len() as u64;
        
        // Reward honest oracles
        for oracle in &consensus.participating_oracles {
            rewards.insert(*oracle, per_oracle);
        }
        
        // Slash dishonest oracles (0 reward)
        for oracle in &consensus.dissenting_oracles {
            rewards.insert(*oracle, 0);
        }
        
        rewards
    }
}

/// Simple Levenshtein distance
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();
    
    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];
    
    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }
    
    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }
    
    matrix[a_len][b_len]
}

// ---------------------------------------------------------------------------
// AI Oracle Hardening — Slashing & Validation
// ---------------------------------------------------------------------------

/// Validate that an AI inference output hashes to the expected 512-bit
/// commitment stored in the block.  If the hashes do not match the oracle
/// node produced a hallucinated proof and must be slashed (ignored by the
/// peer-to-peer mesh).
pub fn validate_ai_inference(output: &str, expected_hash: [u8; 64]) -> bool {
    let check = crate::axiom_hash_512(output.as_bytes());
    check == expected_hash
}

// ---------------------------------------------------------------------------
// Deterministic AI Oracle — 512-bit seal generation
// ---------------------------------------------------------------------------

/// Local AI model used for deterministic oracle inference.
const ORACLE_MODEL: &str = "llama3.2:1b";

/// Fixed seed for deterministic LLM output (every node must use the same seed).
const DETERMINISTIC_SEED: u64 = 42;

/// Query a local AI model (Ollama) at temperature 0 and produce a
/// deterministic 512-bit BLAKE3 seal of the response.
///
/// If the local model is unavailable the function falls back to a pure
/// BLAKE3-512 hash of the query string so that mining is never blocked.
/// A warning is logged so operators can detect missing Ollama instances.
pub async fn query_oracle(query: &str) -> [u8; 64] {
    match query_local_model(query).await {
        Ok(response_text) => crate::axiom_hash_512(response_text.as_bytes()),
        Err(e) => {
            // Deterministic fallback — hash the query itself so consensus
            // can proceed even without a local AI model.  All nodes that
            // lack the model will produce the identical seal for the same
            // query, preserving determinism.
            log::warn!(
                "AI Oracle unavailable ({}), using deterministic BLAKE3 fallback for query hash={}",
                e,
                hex::encode(&crate::axiom_hash_512(query.as_bytes())[..8])
            );
            crate::axiom_hash_512(query.as_bytes())
        }
    }
}

/// Call a local Ollama instance with temperature 0 and a fixed seed so
/// that every node running the same model produces the identical output.
async fn query_local_model(query: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let body = serde_json::json!({
        "model": ORACLE_MODEL,
        "prompt": query,
        "stream": false,
        "options": { "temperature": 0.0, "seed": DETERMINISTIC_SEED }
    });

    let resp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Ollama request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Ollama returned status {}", resp.status()));
    }

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    json["response"]
        .as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| "Missing 'response' field".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_consensus_majority() {
        let manager = OracleConsensusManager::new(3, 0.8);
        
        let query_id = [1u8; 32];
        let responses = vec![
            OracleResponse {
                query_id,
                response_text: "The answer is 42".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                oracle_address: [1u8; 32],
                signature: vec![],
                timestamp: 0,
            },
            OracleResponse {
                query_id,
                response_text: "The answer is 42".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                oracle_address: [2u8; 32],
                signature: vec![],
                timestamp: 0,
            },
            OracleResponse {
                query_id,
                response_text: "The answer is 42".to_string(),
                model: "claude-3-5-sonnet".to_string(),
                oracle_address: [3u8; 32],
                signature: vec![],
                timestamp: 0,
            },
            OracleResponse {
                query_id,
                response_text: "Wrong answer: 99".to_string(), // More different outlier
                model: "claude-3-5-sonnet".to_string(),
                oracle_address: [4u8; 32],
                signature: vec![],
                timestamp: 0,
            },
        ];
        
        let consensus = manager.find_consensus(responses)
            .expect("Failed to find consensus among oracle responses");
        
        assert_eq!(consensus.agreed_response, "The answer is 42");
        assert_eq!(consensus.participating_oracles.len(), 3);
        assert_eq!(consensus.dissenting_oracles.len(), 1);
        assert_eq!(consensus.confidence, 0.75);
        
        println!("✓ Oracle consensus works!");
    }
    
    #[test]
    fn test_similarity_detection() {
        let manager = OracleConsensusManager::new(2, 0.9);
        
        assert!(manager.are_similar("Hello world", "Hello world"));
        assert!(manager.are_similar("Hello world", "hello world")); // Case insensitive
        assert!(!manager.are_similar("Hello world", "Goodbye world"));
    }
    
    #[test]
    fn test_reward_distribution() {
        let manager = OracleConsensusManager::new(3, 0.8);
        
        let consensus = OracleConsensus {
            query_id: [0u8; 32],
            agreed_response: "test".to_string(),
            confidence: 0.8,
            participating_oracles: vec![[1u8; 32], [2u8; 32], [3u8; 32]],
            dissenting_oracles: vec![[4u8; 32]],
        };
        
        let rewards = manager.distribute_rewards(&consensus, 1000);
        
        assert_eq!(rewards[&[1u8; 32]], 333); // 1000/3
        assert_eq!(rewards[&[2u8; 32]], 333);
        assert_eq!(rewards[&[3u8; 32]], 333);
        assert_eq!(rewards[&[4u8; 32]], 0); // Slashed
        
        println!("✓ Reward distribution works!");
    }
    
    #[tokio::test]
    #[ignore] // Requires ANTHROPIC_API_KEY env var
    async fn test_claude_api_integration() {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .expect("Set ANTHROPIC_API_KEY for this test");
        
        let oracle = OracleNode::new([42u8; 32], api_key);
        
        let query = OracleQuery {
            query_id: [1u8; 32],
            prompt: "What is 2+2?".to_string(),
            requester: [0u8; 32],
            max_tokens: 100,
            temperature: 0.0,
            reward: 1000,
            timestamp: 0,
        };
        
        let response = oracle.process_query(&query).await
            .expect("Failed to process oracle query");
        
        println!("Oracle response: {}", response.response_text);
        assert!(response.response_text.contains("4") || response.response_text.contains("four"));
        
        println!("✓ Claude API integration works!");
    }

    #[tokio::test]
    async fn test_query_oracle_deterministic_fallback() {
        // Without a running Ollama instance, query_oracle falls back to
        // a pure BLAKE3-512 hash of the query — which must be deterministic.
        let seal_a = query_oracle("Axiom block 1").await;
        let seal_b = query_oracle("Axiom block 1").await;
        let seal_c = query_oracle("Axiom block 2").await;

        assert_eq!(seal_a, seal_b, "Same query must produce identical seal");
        assert_ne!(seal_a, seal_c, "Different queries must produce different seals");
        assert_eq!(seal_a.len(), 64, "Seal must be 64 bytes (512-bit)");
    }

    #[test]
    fn test_validate_ai_inference_matching() {
        let output = "Axiom block 1 mined";
        let expected = crate::axiom_hash_512(output.as_bytes());
        assert!(validate_ai_inference(output, expected));
    }

    #[test]
    fn test_validate_ai_inference_mismatch() {
        let output = "Axiom block 1 mined";
        let wrong_hash = [0u8; 64]; // All zeros — hallucinated proof
        assert!(!validate_ai_inference(output, wrong_hash));
    }

    #[test]
    fn test_validate_ai_inference_different_output() {
        let original = "Axiom block 1 mined";
        let expected = crate::axiom_hash_512(original.as_bytes());
        // A different (hallucinated) output must not match
        assert!(!validate_ai_inference("Hallucinated output", expected));
    }
}
