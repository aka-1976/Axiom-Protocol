// src/ai/oracle.rs - AI Oracle Network for AXIOM Protocol
// Decentralized LLM inference with consensus and verification

use serde::{Serialize, Deserialize};
use reqwest;
use sha2::{Sha256, Digest};
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
}

impl OracleNode {
    pub fn new(address: [u8; 32], api_key: String) -> Self {
        Self {
            address,
            api_key,
            model: "claude-3-5-sonnet-20241022".to_string(),
        }
    }
    
    /// Process oracle query using Claude API
    pub async fn process_query(&self, query: &OracleQuery) -> Result<OracleResponse, String> {
        println!("Oracle {}: Processing query {}", 
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
                    eprintln!("⚠️  Failed to get oracle timestamp: {}", e);
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
    
    /// Sign oracle response (simplified - use Ed25519 in production)
    fn sign_response(&self, query_id: &[u8; 32], response: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(query_id);
        hasher.update(response.as_bytes());
        hasher.update(self.address);
        hasher.finalize().to_vec()
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
        // Simplified similarity - use embeddings in production
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
}
