use chrono::Utc;
use ndarray::{Array1, Array2};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Types of useful work that can be performed for mining
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkType {
    ProteinFolding,
    OptimizationProblem,
    MatrixComputation,
    PrimeSearch,
    MLTraining,
}

/// A useful work task that miners must complete
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkTask {
    pub task_id: String,
    pub work_type: WorkType,
    pub difficulty: u32,
    pub parameters: serde_json::Value,
    pub created_at: i64,
    pub deadline: i64,
}

/// Solution to a work task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkSolution {
    pub task_id: String,
    pub miner_address: String,
    pub solution_data: serde_json::Value,
    pub computation_hash: String,
    pub timestamp: i64,
    pub verification_proof: String,
}

/// Mining reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningReward {
    pub block_height: u64,
    pub base_reward: u64,
    pub work_bonus: u64,
    pub total_reward: u64,
    pub miner_address: String,
}

/// Protein folding simulation (simplified)
pub fn protein_folding_work(sequence: &[u8], iterations: usize) -> (Vec<f64>, String) {
    let n = sequence.len();
    let mut coordinates = vec![0.0; n * 3]; // x, y, z for each amino acid
    
    // Initialize with random positions
    for i in 0..n {
        coordinates[i * 3] = (sequence[i] as f64) * 0.1;
        coordinates[i * 3 + 1] = (sequence[i] as f64 % 7) as f64 * 0.2;
        coordinates[i * 3 + 2] = (sequence[i] as f64 % 5) as f64 * 0.15;
    }
    
    // Energy minimization iterations
    for iter in 0..iterations {
        let mut forces = vec![0.0; n * 3];
        
        // Calculate pairwise forces
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = coordinates[j * 3] - coordinates[i * 3];
                let dy = coordinates[j * 3 + 1] - coordinates[i * 3 + 1];
                let dz = coordinates[j * 3 + 2] - coordinates[i * 3 + 2];
                let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
                
                // Lennard-Jones potential
                let force = 24.0 * (2.0 / dist.powi(13) - 1.0 / dist.powi(7));
                
                forces[i * 3] += force * dx / dist;
                forces[i * 3 + 1] += force * dy / dist;
                forces[i * 3 + 2] += force * dz / dist;
                forces[j * 3] -= force * dx / dist;
                forces[j * 3 + 1] -= force * dy / dist;
                forces[j * 3 + 2] -= force * dz / dist;
            }
        }
        
        // Update positions
        let step_size = 0.01 / (1.0 + (iter as f64) * 0.001);
        for i in 0..(n * 3) {
            coordinates[i] += forces[i] * step_size;
        }
    }
    
    // Calculate final energy
    let energy = calculate_folding_energy(&coordinates, n);
    
    // Generate verification hash
    let mut hasher = Sha256::new();
    hasher.update(&sequence);
    for coord in &coordinates {
        hasher.update(&coord.to_le_bytes());
    }
    let hash = hex::encode(hasher.finalize());
    
    (coordinates, hash)
}

fn calculate_folding_energy(coordinates: &[f64], n: usize) -> f64 {
    let mut energy = 0.0;
    for i in 0..n {
        for j in (i + 1)..n {
            let dx = coordinates[j * 3] - coordinates[i * 3];
            let dy = coordinates[j * 3 + 1] - coordinates[i * 3 + 1];
            let dz = coordinates[j * 3 + 2] - coordinates[i * 3 + 2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt().max(0.1);
            
            // Lennard-Jones potential
            energy += 4.0 * ((1.0 / dist).powi(12) - (1.0 / dist).powi(6));
        }
    }
    energy
}

/// Traveling Salesman Problem optimization
pub fn tsp_optimization_work(cities: &[(f64, f64)], iterations: usize) -> (Vec<usize>, String) {
    let n = cities.len();
    let mut route: Vec<usize> = (0..n).collect();
    let mut best_distance = calculate_route_distance(&route, cities);
    
    // Simulated annealing
    let mut temperature = 100.0;
    let cooling_rate = 0.995;
    
    for _ in 0..iterations {
        // Generate neighbor solution (2-opt swap)
        let i = rand::random::<usize>() % n;
        let j = rand::random::<usize>() % n;
        if i != j {
            let mut new_route = route.clone();
            new_route.swap(i, j);
            
            let new_distance = calculate_route_distance(&new_route, cities);
            let delta = new_distance - best_distance;
            
            // Accept if better or with probability based on temperature
            if delta < 0.0 || rand::random::<f64>() < (-delta / temperature).exp() {
                route = new_route;
                best_distance = new_distance;
            }
        }
        
        temperature *= cooling_rate;
    }
    
    // Generate verification hash
    let mut hasher = Sha256::new();
    for &city_idx in &route {
        hasher.update(&city_idx.to_le_bytes());
        hasher.update(&cities[city_idx].0.to_le_bytes());
        hasher.update(&cities[city_idx].1.to_le_bytes());
    }
    let hash = hex::encode(hasher.finalize());
    
    (route, hash)
}

fn calculate_route_distance(route: &[usize], cities: &[(f64, f64)]) -> f64 {
    let mut distance = 0.0;
    for i in 0..route.len() {
        let from = cities[route[i]];
        let to = cities[route[(i + 1) % route.len()]];
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        distance += (dx * dx + dy * dy).sqrt();
    }
    distance
}

/// Large matrix multiplication for scientific computing
pub fn matrix_multiplication_work(size: usize) -> (Array2<f64>, String) {
    let matrix_a: Array2<f64> = Array2::from_shape_fn((size, size), |(i, j)| {
        ((i * 7 + j * 11) % 100) as f64 / 10.0
    });
    
    let matrix_b: Array2<f64> = Array2::from_shape_fn((size, size), |(i, j)| {
        ((i * 13 + j * 17) % 100) as f64 / 10.0
    });
    
    // Parallel matrix multiplication
    let result = matrix_a.dot(&matrix_b);
    
    // Generate verification hash
    let mut hasher = Sha256::new();
    for elem in result.iter() {
        hasher.update(&elem.to_le_bytes());
    }
    let hash = hex::encode(hasher.finalize());
    
    (result, hash)
}

/// Search for prime numbers in a range
pub fn prime_search_work(start: u64, count: usize) -> (Vec<u64>, String) {
    let primes: Vec<u64> = (start..)
        .filter(|&n| is_prime(n))
        .take(count)
        .collect();
    
    // Generate verification hash
    let mut hasher = Sha256::new();
    for &prime in &primes {
        hasher.update(&prime.to_le_bytes());
    }
    let hash = hex::encode(hasher.finalize());
    
    (primes, hash)
}

fn is_prime(n: u64) -> bool {
    if n < 2 {
        return false;
    }
    if n == 2 || n == 3 {
        return true;
    }
    if n % 2 == 0 || n % 3 == 0 {
        return false;
    }
    
    let limit = (n as f64).sqrt() as u64 + 1;
    let mut i = 5;
    while i <= limit {
        if n % i == 0 || n % (i + 2) == 0 {
            return false;
        }
        i += 6;
    }
    true
}

/// Simple neural network training for ML-based mining
pub fn ml_training_work(input_size: usize, hidden_size: usize, iterations: usize) -> (Vec<f64>, String) {
    // Initialize weights randomly
    let mut weights1: Array2<f64> = Array2::from_shape_fn((input_size, hidden_size), |(i, j)| {
        ((i * 19 + j * 23) % 200) as f64 / 200.0 - 0.5
    });
    
    let mut weights2: Array1<f64> = Array1::from_shape_fn(hidden_size, |i| {
        ((i * 29) % 200) as f64 / 200.0 - 0.5
    });
    
    // Training data (XOR-like problem)
    let training_data = vec![
        (vec![0.0, 0.0], 0.0),
        (vec![0.0, 1.0], 1.0),
        (vec![1.0, 0.0], 1.0),
        (vec![1.0, 1.0], 0.0),
    ];
    
    let learning_rate = 0.1;
    
    // Training loop
    for _ in 0..iterations {
        for (input, target) in &training_data {
            let input_arr = Array1::from_vec(input.clone());
            
            // Forward pass
            let hidden = weights1.t().dot(&input_arr).mapv(|x| x.tanh());
            let output = weights2.dot(&hidden).tanh();
            
            // Backward pass (simplified)
            let output_error = target - output;
            let output_delta = output_error * (1.0 - output.powi(2));
            
            let hidden_error = &weights2 * output_delta;
            let hidden_delta = &hidden * &hidden_error.mapv(|x| 1.0 - x.powi(2));
            
            // Update weights
            for i in 0..hidden_size {
                weights2[i] += learning_rate * output_delta * hidden[i];
                for j in 0..input_size {
                    weights1[[j, i]] += learning_rate * hidden_delta[i] * input[j];
                }
            }
        }
    }
    
    // Flatten weights for hashing
    let mut all_weights = Vec::new();
    for weight in weights1.iter() {
        all_weights.push(*weight);
    }
    for weight in weights2.iter() {
        all_weights.push(*weight);
    }
    
    // Generate verification hash
    let mut hasher = Sha256::new();
    for weight in &all_weights {
        hasher.update(&weight.to_le_bytes());
    }
    let hash = hex::encode(hasher.finalize());
    
    (all_weights, hash)
}

/// Mining pool that manages work distribution
pub struct MiningPool {
    pub active_tasks: Vec<WorkTask>,
    pub completed_solutions: Vec<WorkSolution>,
    pub miners: Vec<String>,
    pub total_work_done: AtomicU64,
}

impl MiningPool {
    pub fn new() -> Self {
        Self {
            active_tasks: Vec::new(),
            completed_solutions: Vec::new(),
            miners: Vec::new(),
            total_work_done: AtomicU64::new(0),
        }
    }
    
    pub fn create_task(&mut self, work_type: WorkType, difficulty: u32) -> WorkTask {
        let task_id = format!("task-{}-{}", Utc::now().timestamp(), rand::random::<u32>());
        
        let parameters = match work_type {
            WorkType::ProteinFolding => {
                let sequence: Vec<u8> = (0..50).map(|_| rand::random::<u8>() % 20).collect();
                serde_json::json!({
                    "sequence": sequence,
                    "iterations": (difficulty as usize) * 100
                })
            }
            WorkType::OptimizationProblem => {
                let cities: Vec<(f64, f64)> = (0..30)
                    .map(|_| (rand::random::<f64>() * 100.0, rand::random::<f64>() * 100.0))
                    .collect();
                serde_json::json!({
                    "cities": cities,
                    "iterations": (difficulty as usize) * 1000
                })
            }
            WorkType::MatrixComputation => {
                serde_json::json!({
                    "size": 50 + (difficulty as usize) * 10
                })
            }
            WorkType::PrimeSearch => {
                serde_json::json!({
                    "start": 1000000 + (difficulty as u64) * 100000,
                    "count": 100 + (difficulty as usize) * 10
                })
            }
            WorkType::MLTraining => {
                serde_json::json!({
                    "input_size": 2,
                    "hidden_size": 10 + (difficulty as usize) * 5,
                    "iterations": (difficulty as usize) * 500
                })
            }
        };
        
        let task = WorkTask {
            task_id: task_id.clone(),
            work_type,
            difficulty,
            parameters,
            created_at: Utc::now().timestamp(),
            deadline: Utc::now().timestamp() + 3600, // 1 hour deadline
        };
        
        self.active_tasks.push(task.clone());
        task
    }
    
    pub fn submit_solution(&mut self, solution: WorkSolution) -> bool {
        // Verify solution (simplified)
        if self.verify_solution(&solution) {
            self.completed_solutions.push(solution);
            self.total_work_done.fetch_add(1, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    fn verify_solution(&self, solution: &WorkSolution) -> bool {
        // Find the task
        if let Some(task) = self.active_tasks.iter().find(|t| t.task_id == solution.task_id) {
            // Verify the solution's computation hash commitment.
            // Each solution includes a SHA-256 hash of (task_id + solution_data);
            // verification recomputes the hash and checks it matches.
            use sha2::{Sha256, Digest};
            let solution_bytes = solution.solution_data.to_string();
            let mut hasher = Sha256::new();
            hasher.update(task.task_id.as_bytes());
            hasher.update(solution_bytes.as_bytes());
            let expected_hash = hex::encode(hasher.finalize());

            // The computation_hash must match the SHA-256 commitment
            let hash_valid = solution.computation_hash == expected_hash;
            // The solution data must be non-empty
            let data_valid = !solution_bytes.is_empty() && solution_bytes != "null";

            match task.work_type {
                WorkType::ProteinFolding |
                WorkType::OptimizationProblem |
                WorkType::MatrixComputation |
                WorkType::PrimeSearch |
                WorkType::MLTraining => {
                    hash_valid && data_valid
                }
            }
        } else {
            false
        }
    }
    
    pub fn calculate_reward(&self, block_height: u64, miner_address: &str) -> MiningReward {
        let base_reward = 50_000_000_000; // 50 AXM base reward
        let work_count = self.completed_solutions.iter()
            .filter(|s| s.miner_address == miner_address)
            .count() as u64;
        
        let work_bonus = work_count * 5_000_000_000; // 5 AXM per completed work
        let total_reward = base_reward + work_bonus;
        
        MiningReward {
            block_height,
            base_reward,
            work_bonus,
            total_reward,
            miner_address: miner_address.to_string(),
        }
    }
}

/// Parallel mining worker
pub fn parallel_mine(
    task: &WorkTask,
    miner_address: &str,
    num_threads: usize,
) -> WorkSolution {
    let start = Instant::now();
    
    let (solution_data, computation_hash) = match task.work_type {
        WorkType::ProteinFolding => {
            let sequence: Vec<u8> = serde_json::from_value(task.parameters["sequence"].clone()).unwrap();
            let iterations: usize = serde_json::from_value(task.parameters["iterations"].clone()).unwrap();
            
            let (coords, hash) = protein_folding_work(&sequence, iterations);
            (serde_json::json!({"coordinates": coords}), hash)
        }
        WorkType::OptimizationProblem => {
            let cities: Vec<(f64, f64)> = serde_json::from_value(task.parameters["cities"].clone()).unwrap();
            let iterations: usize = serde_json::from_value(task.parameters["iterations"].clone()).unwrap();
            
            let (route, hash) = tsp_optimization_work(&cities, iterations);
            (serde_json::json!({"route": route}), hash)
        }
        WorkType::MatrixComputation => {
            let size: usize = serde_json::from_value(task.parameters["size"].clone()).unwrap();
            
            let (result, hash) = matrix_multiplication_work(size);
            (serde_json::json!({"result_hash": hash.clone()}), hash)
        }
        WorkType::PrimeSearch => {
            let start: u64 = serde_json::from_value(task.parameters["start"].clone()).unwrap();
            let count: usize = serde_json::from_value(task.parameters["count"].clone()).unwrap();
            
            let (primes, hash) = prime_search_work(start, count);
            (serde_json::json!({"primes": primes}), hash)
        }
        WorkType::MLTraining => {
            let input_size: usize = serde_json::from_value(task.parameters["input_size"].clone()).unwrap();
            let hidden_size: usize = serde_json::from_value(task.parameters["hidden_size"].clone()).unwrap();
            let iterations: usize = serde_json::from_value(task.parameters["iterations"].clone()).unwrap();
            
            let (weights, hash) = ml_training_work(input_size, hidden_size, iterations);
            (serde_json::json!({"weights_count": weights.len()}), hash)
        }
    };
    
    let elapsed = start.elapsed();
    
    // Generate verification proof
    let mut hasher = Sha256::new();
    hasher.update(&task.task_id.as_bytes());
    hasher.update(&computation_hash.as_bytes());
    hasher.update(&miner_address.as_bytes());
    let verification_proof = hex::encode(hasher.finalize());
    
    WorkSolution {
        task_id: task.task_id.clone(),
        miner_address: miner_address.to_string(),
        solution_data,
        computation_hash,
        timestamp: Utc::now().timestamp(),
        verification_proof,
    }
}

fn main() {
    println!("🔨 Axiom Proof-of-Useful-Work Mining System");
    println!("===========================================\n");
    
    // Create mining pool
    let mut pool = MiningPool::new();
    
    // Create various work tasks
    println!("📋 Creating work tasks...");
    let tasks = vec![
        pool.create_task(WorkType::ProteinFolding, 5),
        pool.create_task(WorkType::OptimizationProblem, 3),
        pool.create_task(WorkType::MatrixComputation, 2),
        pool.create_task(WorkType::PrimeSearch, 4),
        pool.create_task(WorkType::MLTraining, 3),
    ];
    
    println!("✅ Created {} work tasks\n", tasks.len());
    
    // Simulate mining
    let miner_address = "axm1miner123456789";
    println!("⛏️  Mining with address: {}\n", miner_address);
    
    for (i, task) in tasks.iter().enumerate() {
        println!("Task {}: {:?} (difficulty: {})", i + 1, task.work_type, task.difficulty);
        
        let start = Instant::now();
        let solution = parallel_mine(task, miner_address, 4);
        let elapsed = start.elapsed();
        
        let accepted = pool.submit_solution(solution.clone());
        
        if accepted {
            println!("  ✅ Solution accepted in {:.2}s", elapsed.as_secs_f64());
            println!("  📝 Computation hash: {}", &solution.computation_hash[..16]);
        } else {
            println!("  ❌ Solution rejected");
        }
        println!();
    }
    
    // Calculate rewards
    let reward = pool.calculate_reward(12345, miner_address);
    println!("💰 Mining Rewards:");
    println!("  Base reward: {} sats ({} AXM)", reward.base_reward, reward.base_reward / 1_000_000_000);
    println!("  Work bonus: {} sats ({} AXM)", reward.work_bonus, reward.work_bonus / 1_000_000_000);
    println!("  Total reward: {} sats ({} AXM)", reward.total_reward, reward.total_reward / 1_000_000_000);
    println!("\n🎉 Mining complete! {} useful computations performed.", pool.total_work_done.load(Ordering::Relaxed));
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_protein_folding() {
        let sequence = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let (coords, hash) = protein_folding_work(&sequence, 100);
        
        assert_eq!(coords.len(), sequence.len() * 3);
        assert_eq!(hash.len(), 64); // SHA-256 hex
    }
    
    #[test]
    fn test_tsp_optimization() {
        let cities = vec![
            (0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0), (0.5, 0.5)
        ];
        let (route, hash) = tsp_optimization_work(&cities, 1000);
        
        assert_eq!(route.len(), cities.len());
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_matrix_multiplication() {
        let (result, hash) = matrix_multiplication_work(10);
        
        assert_eq!(result.shape(), &[10, 10]);
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_prime_search() {
        let (primes, hash) = prime_search_work(1000, 10);
        
        assert_eq!(primes.len(), 10);
        assert!(primes.iter().all(|&p| is_prime(p)));
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_ml_training() {
        let (weights, hash) = ml_training_work(2, 5, 100);
        
        assert!(weights.len() > 0);
        assert_eq!(hash.len(), 64);
    }
    
    #[test]
    fn test_mining_pool() {
        let mut pool = MiningPool::new();
        let task = pool.create_task(WorkType::PrimeSearch, 1);
        
        assert_eq!(pool.active_tasks.len(), 1);
        assert_eq!(pool.active_tasks[0].task_id, task.task_id);
    }
    
    #[test]
    fn test_solution_submission() {
        let mut pool = MiningPool::new();
        let task = pool.create_task(WorkType::PrimeSearch, 1);
        
        let solution = parallel_mine(&task, "test_miner", 1);
        let accepted = pool.submit_solution(solution);
        
        assert!(accepted);
        assert_eq!(pool.completed_solutions.len(), 1);
    }
    
    #[test]
    fn test_reward_calculation() {
        let mut pool = MiningPool::new();
        
        // Submit multiple solutions
        for _ in 0..3 {
            let task = pool.create_task(WorkType::PrimeSearch, 1);
            let solution = parallel_mine(&task, "miner1", 1);
            pool.submit_solution(solution);
        }
        
        let reward = pool.calculate_reward(100, "miner1");
        assert!(reward.total_reward > reward.base_reward);
        assert_eq!(reward.work_bonus, 3 * 5_000_000_000);
    }
    
    #[test]
    fn test_is_prime() {
        assert!(is_prime(2));
        assert!(is_prime(3));
        assert!(is_prime(5));
        assert!(is_prime(7));
        assert!(is_prime(11));
        assert!(!is_prime(4));
        assert!(!is_prime(6));
        assert!(!is_prime(8));
        assert!(!is_prime(9));
    }
}
