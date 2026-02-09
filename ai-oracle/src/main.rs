use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Oracle request from smart contracts or users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleRequest {
    pub request_id: String,
    pub query: String,
    pub model: String, // "gpt-4", "claude", "llama", etc.
    pub requester: String,
    pub timestamp: i64,
    pub stake: u64,
}

/// Oracle response with aggregated answers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleResponse {
    pub request_id: String,
    pub answer: String,
    pub confidence: f64,
    pub providers: Vec<ProviderResponse>,
    pub consensus_hash: String,
    pub finalized_at: i64,
}

/// Individual provider's response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub request_id: String,
    pub provider_id: String,
    pub answer: String,
    pub signature: String,
    pub stake_weight: u64,
    pub submitted_at: i64,
}

/// Oracle provider registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OracleProvider {
    pub provider_id: String,
    pub endpoint: String,
    pub models: Vec<String>,
    pub stake: u64,
    pub reputation_score: f64,
    pub total_requests: u64,
    pub successful_requests: u64,
}

/// State of the oracle network
#[derive(Debug, Clone)]
pub struct OracleState {
    pub requests: HashMap<String, OracleRequest>,
    pub responses: HashMap<String, Vec<ProviderResponse>>,
    pub finalized: HashMap<String, OracleResponse>,
    pub providers: HashMap<String, OracleProvider>,
}

impl OracleState {
    fn new() -> Self {
        Self {
            requests: HashMap::new(),
            responses: HashMap::new(),
            finalized: HashMap::new(),
            providers: HashMap::new(),
        }
    }
}

type SharedState = Arc<Mutex<OracleState>>;

/// Submit a new oracle request
async fn submit_request(
    data: web::Json<OracleRequest>,
    state: web::Data<SharedState>,
) -> impl Responder {
    let mut oracle_state = state.lock().unwrap();
    let request = data.into_inner();
    
    println!("📝 New oracle request: {} - {}", request.request_id, request.query);
    
    oracle_state.requests.insert(request.request_id.clone(), request.clone());
    oracle_state.responses.insert(request.request_id.clone(), Vec::new());
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "accepted",
        "request_id": request.request_id,
        "estimated_time": "30s"
    }))
}

/// Provider submits a response
async fn submit_response(
    data: web::Json<ProviderResponse>,
    state: web::Data<SharedState>,
) -> impl Responder {
    let mut oracle_state = state.lock().unwrap();
    let response = data.into_inner();
    
    println!("🔮 Provider response: {} - {}", response.provider_id, &response.answer[..50.min(response.answer.len())]);
    
    if let Some(responses) = oracle_state.responses.get_mut(&response.request_id) {
        responses.push(response.clone());
        
        // Check if we have enough responses to reach consensus
        if responses.len() >= 3 {
            let consensus = reach_consensus(responses);
            let finalized = OracleResponse {
                request_id: response.request_id.clone(),
                answer: consensus.answer,
                confidence: consensus.confidence,
                providers: responses.clone(),
                consensus_hash: consensus.hash,
                finalized_at: Utc::now().timestamp(),
            };
            
            oracle_state.finalized.insert(response.request_id.clone(), finalized);
            
            return HttpResponse::Ok().json(serde_json::json!({
                "status": "finalized",
                "consensus_reached": true
            }));
        }
        
        HttpResponse::Ok().json(serde_json::json!({
            "status": "accepted",
            "responses_count": responses.len(),
            "consensus_threshold": 3
        }))
    } else {
        HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Request not found"
        }))
    }
}

/// Get oracle response
async fn get_response(
    request_id: web::Path<String>,
    state: web::Data<SharedState>,
) -> impl Responder {
    let oracle_state = state.lock().unwrap();
    
    if let Some(response) = oracle_state.finalized.get(request_id.as_str()) {
        HttpResponse::Ok().json(response)
    } else if let Some(responses) = oracle_state.responses.get(request_id.as_str()) {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "pending",
            "responses_count": responses.len(),
            "consensus_threshold": 3
        }))
    } else {
        HttpResponse::NotFound().json(serde_json::json!({
            "error": "Request not found"
        }))
    }
}

/// Register a new oracle provider
async fn register_provider(
    data: web::Json<OracleProvider>,
    state: web::Data<SharedState>,
) -> impl Responder {
    let mut oracle_state = state.lock().unwrap();
    let provider = data.into_inner();
    
    println!("🤖 New provider registered: {} - {:?}", provider.provider_id, provider.models);
    
    oracle_state.providers.insert(provider.provider_id.clone(), provider.clone());
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "registered",
        "provider_id": provider.provider_id
    }))
}

/// List all providers
async fn list_providers(state: web::Data<SharedState>) -> impl Responder {
    let oracle_state = state.lock().unwrap();
    let providers: Vec<&OracleProvider> = oracle_state.providers.values().collect();
    HttpResponse::Ok().json(providers)
}

/// Get network statistics
async fn get_stats(state: web::Data<SharedState>) -> impl Responder {
    let oracle_state = state.lock().unwrap();
    
    let stats = serde_json::json!({
        "total_requests": oracle_state.requests.len(),
        "finalized_responses": oracle_state.finalized.len(),
        "pending_requests": oracle_state.requests.len() - oracle_state.finalized.len(),
        "registered_providers": oracle_state.providers.len(),
        "total_stake": oracle_state.providers.values().map(|p| p.stake).sum::<u64>(),
    });
    
    HttpResponse::Ok().json(stats)
}

/// Reach consensus from multiple provider responses
fn reach_consensus(responses: &[ProviderResponse]) -> ConsensusResult {
    // Simple voting mechanism with stake weighting
    let mut answer_weights: HashMap<String, u64> = HashMap::new();
    
    for response in responses {
        *answer_weights.entry(response.answer.clone()).or_insert(0) += response.stake_weight;
    }
    
    // Find answer with highest stake weight
    let (best_answer, total_weight) = answer_weights
        .iter()
        .max_by_key(|(_, weight)| *weight)
        .map(|(ans, weight)| (ans.clone(), *weight))
        .unwrap_or_else(|| (responses[0].answer.clone(), 0));
    
    let total_stake: u64 = responses.iter().map(|r| r.stake_weight).sum();
    let confidence = if total_stake > 0 {
        total_weight as f64 / total_stake as f64
    } else {
        0.0
    };
    
    // Generate consensus hash
    let mut hasher = Sha256::new();
    hasher.update(best_answer.as_bytes());
    let hash = hex::encode(hasher.finalize());
    
    ConsensusResult {
        answer: best_answer,
        confidence,
        hash,
    }
}

struct ConsensusResult {
    answer: String,
    confidence: f64,
    hash: String,
}

/// Health check endpoint
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "service": "axiom-ai-oracle",
        "version": "1.0.0"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();
    
    let state = Arc::new(Mutex::new(OracleState::new()));
    
    // Initialize with some demo providers
    println!("🌐 Axiom AI Oracle Network starting...");
    println!("   Providers register via POST /api/provider/register");
    println!("🔗 API listening on http://0.0.0.0:8081");
    
    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
        
        App::new()
            .wrap(cors)
            .app_data(web::Data::new(state.clone()))
            .route("/health", web::get().to(health))
            .route("/api/request", web::post().to(submit_request))
            .route("/api/response", web::post().to(submit_response))
            .route("/api/response/{request_id}", web::get().to(get_response))
            .route("/api/provider/register", web::post().to(register_provider))
            .route("/api/providers", web::get().to(list_providers))
            .route("/api/stats", web::get().to(get_stats))
    })
    .bind("0.0.0.0:8081")?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_oracle_state_creation() {
        let state = OracleState::new();
        assert_eq!(state.requests.len(), 0);
        assert_eq!(state.providers.len(), 0);
    }
    
    #[test]
    fn test_consensus_unanimous() {
        let responses = vec![
            ProviderResponse {
                request_id: "req-001".to_string(),
                provider_id: "p1".to_string(),
                answer: "42".to_string(),
                signature: "sig1".to_string(),
                stake_weight: 100,
                submitted_at: 0,
            },
            ProviderResponse {
                request_id: "req-001".to_string(),
                provider_id: "p2".to_string(),
                answer: "42".to_string(),
                signature: "sig2".to_string(),
                stake_weight: 100,
                submitted_at: 0,
            },
            ProviderResponse {
                request_id: "req-001".to_string(),
                provider_id: "p3".to_string(),
                answer: "42".to_string(),
                signature: "sig3".to_string(),
                stake_weight: 100,
                submitted_at: 0,
            },
        ];
        
        let consensus = reach_consensus(&responses);
        assert_eq!(consensus.answer, "42");
        assert_eq!(consensus.confidence, 1.0);
    }
    
    #[test]
    fn test_consensus_majority() {
        let responses = vec![
            ProviderResponse {
                request_id: "req-002".to_string(),
                provider_id: "p1".to_string(),
                answer: "Yes".to_string(),
                signature: "sig1".to_string(),
                stake_weight: 100,
                submitted_at: 0,
            },
            ProviderResponse {
                request_id: "req-002".to_string(),
                provider_id: "p2".to_string(),
                answer: "Yes".to_string(),
                signature: "sig2".to_string(),
                stake_weight: 100,
                submitted_at: 0,
            },
            ProviderResponse {
                request_id: "req-002".to_string(),
                provider_id: "p3".to_string(),
                answer: "No".to_string(),
                signature: "sig3".to_string(),
                stake_weight: 50,
                submitted_at: 0,
            },
        ];
        
        let consensus = reach_consensus(&responses);
        assert_eq!(consensus.answer, "Yes");
        assert!(consensus.confidence > 0.6);
    }
    
    #[test]
    fn test_consensus_weighted() {
        let responses = vec![
            ProviderResponse {
                request_id: "req-003".to_string(),
                provider_id: "high_stake".to_string(),
                answer: "Alice".to_string(),
                signature: "sig1".to_string(),
                stake_weight: 500,
                submitted_at: 0,
            },
            ProviderResponse {
                request_id: "req-003".to_string(),
                provider_id: "low_stake1".to_string(),
                answer: "Bob".to_string(),
                signature: "sig2".to_string(),
                stake_weight: 50,
                submitted_at: 0,
            },
            ProviderResponse {
                request_id: "req-003".to_string(),
                provider_id: "low_stake2".to_string(),
                answer: "Bob".to_string(),
                signature: "sig3".to_string(),
                stake_weight: 50,
                submitted_at: 0,
            },
        ];
        
        let consensus = reach_consensus(&responses);
        // High stake should win despite being outnumbered
        assert_eq!(consensus.answer, "Alice");
    }
    
    #[test]
    fn test_provider_creation() {
        let provider = OracleProvider {
            provider_id: "test-provider".to_string(),
            endpoint: "https://test.api".to_string(),
            models: vec!["model1".to_string()],
            stake: 1000,
            reputation_score: 0.95,
            total_requests: 100,
            successful_requests: 95,
        };
        
        assert_eq!(provider.provider_id, "test-provider");
        assert_eq!(provider.stake, 1000);
    }
    
    #[test]
    fn test_request_serialization() {
        let request = OracleRequest {
            request_id: "req-001".to_string(),
            query: "What is 2+2?".to_string(),
            model: "gpt-4".to_string(),
            requester: "user123".to_string(),
            timestamp: 1234567890,
            stake: 1000,
        };
        
        let json = serde_json::to_string(&request).unwrap();
        let deserialized: OracleRequest = serde_json::from_str(&json).unwrap();
        
        assert_eq!(request.request_id, deserialized.request_id);
        assert_eq!(request.query, deserialized.query);
    }
}
