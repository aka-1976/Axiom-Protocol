// src/consensus/vdf.rs - Verifiable Delay Function for AXIOM Protocol
// Implements Wesolowski VDF for deterministic, sequential proof-of-time

use num_bigint::BigUint;
use num_traits::One;
use num_integer::Integer;
use serde::{Serialize, Deserialize, Serializer, Deserializer};
use sha2::{Sha256, Digest};
use std::time::{Duration, Instant};

// Custom serialization for BigUint
fn serialize_biguint<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_bytes(&value.to_bytes_be())
}

fn deserialize_biguint<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: Deserializer<'de>,
{
    let bytes = <Vec<u8>>::deserialize(deserializer)?;
    Ok(BigUint::from_bytes_be(&bytes))
}

/// VDF (Verifiable Delay Function) for time-lock consensus
/// Uses Wesolowski construction with RSA modulus
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VDF {
    /// RSA modulus N (product of two large primes - from trusted setup)
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint")]
    pub modulus: BigUint,
    /// Number of sequential squarings (time parameter)
    pub time_param: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VDFProof {
    /// Output: y = x^(2^T) mod N
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint")]
    pub output: BigUint,
    /// Proof: π = x^q mod N where q = floor(2^T / ℓ)
    #[serde(serialize_with = "serialize_biguint", deserialize_with = "deserialize_biguint")]
    pub proof: BigUint,
}

impl VDF {
    /// Create new VDF with RSA-2048 modulus (from trusted setup ceremony)
    pub fn new(modulus: BigUint, time_param: u64) -> Self {
        Self { modulus, time_param }
    }
    
    /// Create VDF with default 2048-bit RSA modulus (RSA-2048 challenge number)
    pub fn with_default_modulus(time_param: u64) -> Self {
        // RSA-2048 challenge number — a 2048-bit semiprime whose factorization is unknown.
        // This is a standard choice for VDF moduli; a multi-party ceremony can produce
        // a custom modulus if the RSA Factoring Challenge is ever broken.
        let modulus = BigUint::parse_bytes(
            b"25195908475657893494027183240048398571429282126204032027777137836043662020707595556264018525880784406918290641249515082189298559149176184502808489120072844992687392807287776735971418347270261896375014971824691165077613379859095700097330459748808428401797429100642458691817195118746121515172654632282216869987549182422433637259085141865462043576798423387184774447920739934236584823824281198163815010674810451660377306056201619676256133844143603833904414952634432190114657544454178424020924616515723350778707749817125772467962926386356373289912154831438167899885040445364023527381951378636564391212010397122822120720357",
            10
        ).expect("Invalid RSA modulus");
        
        Self::new(modulus, time_param)
    }
    
    /// Compute VDF: y = x^(2^T) mod N
    /// This is SLOW by design (sequential squaring)
    /// Takes ~1 hour for T calibrated to target block time
    pub fn compute(&self, input: &[u8]) -> Result<VDFProof, String> {
        // Hash input to get starting point
        let x = self.hash_to_prime(input);
        
        println!("VDF: Starting sequential squaring for {} steps...", self.time_param);
        let start = Instant::now();
        
        // Compute y = x^(2^T) mod N via repeated squaring
        let mut y = x.clone();
        for i in 0..self.time_param {
            y = (&y * &y) % &self.modulus;
            
            if i % 1_000_000 == 0 && i > 0 {
                let elapsed = start.elapsed().as_secs();
                let progress = (i as f64 / self.time_param as f64) * 100.0;
                println!("  VDF: {:.1}% complete ({}/{}), {}s elapsed", 
                    progress, i, self.time_param, elapsed);
            }
        }
        
        let elapsed = start.elapsed();
        println!("VDF: Completed in {:.2}s", elapsed.as_secs_f64());
        
        // Generate proof (Wesolowski)
        let proof = self.generate_proof(&x, &y)?;
        
        Ok(VDFProof {
            output: y,
            proof,
        })
    }
    
    /// Verify VDF proof: Fast! (~100ms even though compute took 1 hour)
    /// Checks if y = x^(2^T) mod N using Wesolowski proof
    pub fn verify(&self, input: &[u8], proof: &VDFProof) -> Result<bool, String> {
        let x = self.hash_to_prime(input);
        let y = &proof.output;
        let pi = &proof.proof;
        
        // Wesolowski verification:
        // 1. Compute challenge ℓ = H(x, y)
        let ell = self.hash_to_prime_challenge(&x, y);
        
        // 2. Compute r = 2^T mod ℓ
        let two = BigUint::from(2u32);
        let exp = two.modpow(&BigUint::from(self.time_param), &ell);
        
        // 3. Check if π^ℓ * x^r ≡ y (mod N)
        let lhs = (pi.modpow(&ell, &self.modulus) 
                    * x.modpow(&exp, &self.modulus)) % &self.modulus;
        
        Ok(lhs == *y)
    }
    
    /// Generate Wesolowski proof: π = x^q mod N where q = floor(2^T / ℓ)
    fn generate_proof(&self, x: &BigUint, y: &BigUint) -> Result<BigUint, String> {
        // Challenge ℓ = H(x, y)
        let ell = self.hash_to_prime_challenge(x, y);
        
        // q = floor(2^T / ℓ)
        let two = BigUint::from(2u32);
        let numerator = two.pow(self.time_param as u32);
        let q = numerator / &ell;
        
        // π = x^q mod N
        let proof = x.modpow(&q, &self.modulus);
        
        Ok(proof)
    }
    
    /// Hash input to prime number in group
    fn hash_to_prime(&self, input: &[u8]) -> BigUint {
        let mut hasher = Sha256::new();
        hasher.update(input);
        let hash = hasher.finalize();
        
        BigUint::from_bytes_be(&hash) % &self.modulus
    }
    
    /// Hash to prime challenge for Wesolowski proof
    fn hash_to_prime_challenge(&self, x: &BigUint, y: &BigUint) -> BigUint {
        let mut hasher = Sha256::new();
        hasher.update(x.to_bytes_be());
        hasher.update(y.to_bytes_be());
        let hash = hasher.finalize();
        
        // Convert to prime via deterministic search from hash value
        let mut candidate = BigUint::from_bytes_be(&hash);
        if candidate.is_even() {
            candidate += BigUint::one();
        }
        
        // Find next prime using Miller-Rabin primality test
        while !Self::is_probable_prime(&candidate, 20) {
            candidate += BigUint::from(2u32);
        }
        candidate
    }

    /// Miller-Rabin probabilistic primality test.
    /// Returns true if `n` is probably prime with error probability < 4^(-rounds).
    /// 20 rounds gives error probability < 2^(-40).
    fn is_probable_prime(n: &BigUint, rounds: u32) -> bool {
        let zero = BigUint::from(0u32);
        let one = BigUint::one();
        let two = BigUint::from(2u32);
        let three = BigUint::from(3u32);

        if *n < two {
            return false;
        }
        if *n == two || *n == three {
            return true;
        }
        if n.is_even() {
            return false;
        }

        // Write n-1 as 2^r * d where d is odd
        let n_minus_one = n - &one;
        let mut d = n_minus_one.clone();
        let mut r = 0u32;
        while d.is_even() {
            d /= &two;
            r += 1;
        }

        // Deterministic witnesses for numbers up to 2^64 are [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37]
        // For larger numbers use additional rounds
        let witnesses: Vec<BigUint> = (0..rounds)
            .map(|i| BigUint::from(2u32 + i))
            .filter(|a| a < n)
            .collect();

        'witness: for a in witnesses {
            let mut x = a.modpow(&d, n);

            if x == one || x == n_minus_one {
                continue 'witness;
            }

            for _ in 0..(r - 1) {
                x = x.modpow(&two, n);
                if x == n_minus_one {
                    continue 'witness;
                }
            }

            return false; // composite
        }

        true // probably prime
    }
    
    /// Calibrate time_param to target a specific wall-clock duration
    /// Run this on representative hardware to determine optimal T
    pub fn calibrate(target_duration: Duration) -> u64 {
        println!("Calibrating VDF for target duration: {:?}", target_duration);
        
        let test_input = b"calibration_test";
        let mut time_param = 1_000_000u64; // Start with 1M steps
        
        loop {
            let vdf = VDF::with_default_modulus(time_param);
            let start = Instant::now();
            
            let _ = vdf.compute(test_input).expect("Calibration failed");
            
            let elapsed = start.elapsed();
            println!("  T={}: took {:?}", time_param, elapsed);
            
            if elapsed >= target_duration {
                println!("Calibration complete: T={} for {:?}", time_param, target_duration);
                return time_param;
            }
            
            // Extrapolate next test
            let ratio = target_duration.as_secs_f64() / elapsed.as_secs_f64();
            time_param = (time_param as f64 * ratio) as u64;
        }
    }
}

/// VDF-based block header
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VDFBlockHeader {
    pub prev_block_hash: [u8; 32],
    pub timestamp: u64,
    pub vdf_input: Vec<u8>,
    pub vdf_proof: VDFProof,
}

impl VDFBlockHeader {
    /// Create new block with VDF proof (miners must compute this!)
    pub fn mine(
        prev_block_hash: [u8; 32],
        timestamp: u64,
        vdf: &VDF,
    ) -> Result<Self, String> {
        // VDF input = H(prev_hash || timestamp)
        let mut hasher = Sha256::new();
        hasher.update(prev_block_hash);
        hasher.update(timestamp.to_le_bytes());
        let vdf_input = hasher.finalize().to_vec();
        
        println!("Mining block with VDF...");
        let vdf_proof = vdf.compute(&vdf_input)?;
        
        Ok(Self {
            prev_block_hash,
            timestamp,
            vdf_input,
            vdf_proof,
        })
    }
    
    /// Verify block VDF proof
    pub fn verify(&self, vdf: &VDF) -> Result<bool, String> {
        // Recompute VDF input
        let mut hasher = Sha256::new();
        hasher.update(self.prev_block_hash);
        hasher.update(self.timestamp.to_le_bytes());
        let expected_input = hasher.finalize().to_vec();
        
        if self.vdf_input != expected_input {
            return Ok(false);
        }
        
        vdf.verify(&self.vdf_input, &self.vdf_proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vdf_compute_verify() {
        // Use small time_param for testing (10k steps ~100ms)
        let vdf = VDF::with_default_modulus(10_000);
        let input = b"axiom_protocol_block_12345";
        
        println!("Computing VDF proof...");
        let proof = vdf.compute(input).expect("VDF computation failed");
        
        println!("Verifying VDF proof...");
        let valid = vdf.verify(input, &proof).expect("VDF verification failed");
        
        assert!(valid, "VDF proof should be valid");
        println!("✓ VDF proof valid!");
    }
    
    #[test]
    fn test_vdf_invalid_proof_fails() {
        let vdf = VDF::with_default_modulus(10_000);
        let input = b"test_input";
        
        let proof = vdf.compute(input).expect("Compute failed");
        
        // Tamper with proof
        let mut bad_proof = proof.clone();
        bad_proof.output += BigUint::one();
        
        let valid = vdf.verify(input, &bad_proof).expect("Verification failed");
        assert!(!valid, "Tampered proof should be invalid");
    }
    
    #[test]
    fn test_vdf_block_mining() {
        let vdf = VDF::with_default_modulus(5_000); // Fast for testing
        let prev_hash = [42u8; 32];
        let timestamp = 1234567890;
        
        println!("Mining VDF block...");
        let block = VDFBlockHeader::mine(prev_hash, timestamp, &vdf)
            .expect("Block mining failed");
        
        println!("Verifying VDF block...");
        let valid = block.verify(&vdf).expect("Block verification failed");
        
        assert!(valid, "Block should be valid");
        println!("✓ VDF block valid!");
    }
    
    #[test]
    #[ignore] // Slow test - run manually
    fn test_vdf_calibration() {
        // Calibrate for 1-minute blocks (for testing)
        let target = Duration::from_secs(60);
        let time_param = VDF::calibrate(target);
        
        println!("Calibrated time_param: {}", time_param);
        assert!(time_param > 0);
    }
}
