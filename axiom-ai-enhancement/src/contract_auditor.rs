// Production Smart Contract Vulnerability Scanner
// Pattern-based detection using EVM bytecode analysis

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum VulnerabilityType {
    ReentrancyAttack,
    IntegerOverflow,
    UnauthorizedAccess,
    UncheckedExternalCall,
    FrontRunning,
    TimestampDependence,
    DelegateCallInjection,
    UnprotectedSelfDestruct,
    UnhandledReverts,
    GasOptimization,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub vuln_type: VulnerabilityType,
    pub severity: u8, // 1-10 scale
    pub location: String,
    pub description: String,
    pub suggested_fix: String,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub contract_hash: String,
    pub overall_score: u8, // 0-100
    pub vulnerabilities: Vec<Vulnerability>,
    pub safe_patterns: Vec<String>,
    pub gas_optimization_tips: Vec<String>,
    pub complexity_score: u32,
}

pub struct ContractAuditor {
    pattern_signatures: HashMap<Vec<u8>, VulnerabilityType>,
    opcode_costs: HashMap<u8, u64>,
}

impl ContractAuditor {
    pub fn new() -> Self {
        Self {
            pattern_signatures: Self::init_vulnerability_patterns(),
            opcode_costs: Self::init_opcode_costs(),
        }
    }

    fn init_vulnerability_patterns() -> HashMap<Vec<u8>, VulnerabilityType> {
        let mut patterns = HashMap::new();

        // Reentrancy: CALL followed by SSTORE
        patterns.insert(vec![0xf1, 0x55], VulnerabilityType::ReentrancyAttack);
        patterns.insert(vec![0xf1, 0x50, 0x55], VulnerabilityType::ReentrancyAttack);

        // Integer overflow: ADD/MUL/SUB without checks
        patterns.insert(vec![0x01, 0x50], VulnerabilityType::IntegerOverflow);
        patterns.insert(vec![0x02, 0x50], VulnerabilityType::IntegerOverflow);
        patterns.insert(vec![0x03, 0x50], VulnerabilityType::IntegerOverflow);

        // Unchecked CALL return value
        patterns.insert(vec![0xf1, 0x50], VulnerabilityType::UncheckedExternalCall);
        patterns.insert(vec![0xf1, 0x00], VulnerabilityType::UncheckedExternalCall);

        // DELEGATECALL
        patterns.insert(vec![0xf4], VulnerabilityType::DelegateCallInjection);

        // SELFDESTRUCT
        patterns.insert(vec![0xff], VulnerabilityType::UnprotectedSelfDestruct);

        // TIMESTAMP usage
        patterns.insert(vec![0x42], VulnerabilityType::TimestampDependence);

        patterns
    }

    fn init_opcode_costs() -> HashMap<u8, u64> {
        let mut costs = HashMap::new();

        // Basic opcodes
        costs.insert(0x00, 0); // STOP
        costs.insert(0x01, 3); // ADD
        costs.insert(0x02, 5); // MUL
        costs.insert(0x03, 3); // SUB
        costs.insert(0x04, 5); // DIV
        costs.insert(0x10, 3); // LT
        costs.insert(0x11, 3); // GT
        costs.insert(0x14, 3); // EQ
        costs.insert(0x15, 3); // ISZERO

        // Memory/Storage opcodes
        costs.insert(0x50, 2); // POP
        costs.insert(0x51, 3); // MLOAD
        costs.insert(0x52, 3); // MSTORE
        costs.insert(0x53, 3); // MSTORE8
        costs.insert(0x54, 800); // SLOAD (expensive!)
        costs.insert(0x55, 20000); // SSTORE (very expensive!)
        costs.insert(0x56, 8); // JUMP
        costs.insert(0x57, 10); // JUMPI

        // Push opcodes
        for i in 0x60..=0x7f {
            costs.insert(i, 3);
        }

        // Dup/Swap opcodes
        for i in 0x80..=0x8f {
            costs.insert(i, 3);
        }
        for i in 0x90..=0x9f {
            costs.insert(i, 3);
        }

        // Call opcodes
        costs.insert(0xf1, 700); // CALL
        costs.insert(0xf2, 700); // CALLCODE
        costs.insert(0xf4, 700); // DELEGATECALL
        costs.insert(0xfa, 700); // STATICCALL

        costs
    }

    /// Main audit function
    pub fn audit_contract(&self, bytecode: &[u8]) -> AuditReport {
        let contract_hash = self.hash_contract(bytecode);

        let mut vulnerabilities = Vec::new();
        let mut safe_patterns = Vec::new();
        let mut gas_tips = Vec::new();

        // 1. Pattern matching for known vulnerabilities
        vulnerabilities.extend(self.detect_known_patterns(bytecode));

        // 2. Data flow analysis
        vulnerabilities.extend(self.analyze_dataflow(bytecode));

        // 3. Control flow analysis
        vulnerabilities.extend(self.analyze_control_flow(bytecode));

        // 4. Access control analysis
        vulnerabilities.extend(self.analyze_access_control(bytecode));

        // 5. Gas optimization detection
        gas_tips = self.detect_gas_inefficiencies(bytecode);

        // 6. Safe pattern recognition
        safe_patterns = self.detect_safe_patterns(bytecode);

        // 7. Calculate complexity
        let complexity_score = self.calculate_complexity(bytecode);

        // Calculate overall security score
        let overall_score = self.calculate_security_score(&vulnerabilities, complexity_score);

        AuditReport {
            contract_hash,
            overall_score,
            vulnerabilities,
            safe_patterns,
            gas_optimization_tips: gas_tips,
            complexity_score,
        }
    }

    fn hash_contract(&self, bytecode: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(bytecode);
        format!("{:x}", hasher.finalize())
    }

    /// Detect known vulnerability patterns
    fn detect_known_patterns(&self, bytecode: &[u8]) -> Vec<Vulnerability> {
        let mut vulnerabilities = Vec::new();
        let mut seen_locations = HashSet::new();

        for i in 0..bytecode.len() {
            for (pattern, vuln_type) in &self.pattern_signatures {
                if i + pattern.len() <= bytecode.len() {
                    if &bytecode[i..i + pattern.len()] == pattern.as_slice() {
                        let location = format!("Offset: 0x{:x}", i);

                        // Avoid duplicate reports at same location
                        if seen_locations.contains(&location) {
                            continue;
                        }
                        seen_locations.insert(location.clone());

                        let vulnerability = match vuln_type {
                            VulnerabilityType::ReentrancyAttack => Vulnerability {
                                vuln_type: vuln_type.clone(),
                                severity: 9,
                                location: location.clone(),
                                description:
                                    "Potential reentrancy vulnerability: external call before state change"
                                        .to_string(),
                                suggested_fix:
                                    "Use checks-effects-interactions pattern or ReentrancyGuard"
                                        .to_string(),
                                confidence: 0.85,
                            },
                            VulnerabilityType::IntegerOverflow => Vulnerability {
                                vuln_type: vuln_type.clone(),
                                severity: 7,
                                location: location.clone(),
                                description: "Arithmetic operation without overflow check".to_string(),
                                suggested_fix: "Use checked arithmetic operations".to_string(),
                                confidence: 0.7,
                            },
                            VulnerabilityType::UncheckedExternalCall => Vulnerability {
                                vuln_type: vuln_type.clone(),
                                severity: 8,
                                location: location.clone(),
                                description: "External call without return value check".to_string(),
                                suggested_fix: "Always verify external call return values".to_string(),
                                confidence: 0.8,
                            },
                            VulnerabilityType::DelegateCallInjection => Vulnerability {
                                vuln_type: vuln_type.clone(),
                                severity: 10,
                                location: location.clone(),
                                description: "DELEGATECALL to potentially untrusted contract".to_string(),
                                suggested_fix: "Whitelist allowed delegate call targets".to_string(),
                                confidence: 0.9,
                            },
                            VulnerabilityType::UnprotectedSelfDestruct => Vulnerability {
                                vuln_type: vuln_type.clone(),
                                severity: 10,
                                location: location.clone(),
                                description: "SELFDESTRUCT without access control".to_string(),
                                suggested_fix: "Add owner-only modifier".to_string(),
                                confidence: 0.95,
                            },
                            VulnerabilityType::TimestampDependence => Vulnerability {
                                vuln_type: vuln_type.clone(),
                                severity: 5,
                                location: location.clone(),
                                description: "Logic depends on block timestamp (manipulable by miners)"
                                    .to_string(),
                                suggested_fix: "Avoid timestamp for critical logic".to_string(),
                                confidence: 0.6,
                            },
                            _ => continue,
                        };

                        vulnerabilities.push(vulnerability);
                    }
                }
            }
        }

        vulnerabilities
    }

    /// Data flow analysis to detect reentrancy
    fn analyze_dataflow(&self, bytecode: &[u8]) -> Vec<Vulnerability> {
        let mut vulnerabilities = Vec::new();
        let mut external_calls = Vec::new();
        let mut storage_writes = Vec::new();

        for (i, &opcode) in bytecode.iter().enumerate() {
            match opcode {
                0xf1 | 0xf2 | 0xf4 | 0xfa => external_calls.push(i), // CALL variants
                0x55 => storage_writes.push(i),                       // SSTORE
                _ => {}
            }
        }

        // Check if storage writes occur after external calls (reentrancy risk)
        for &call_pos in &external_calls {
            for &write_pos in &storage_writes {
                if write_pos > call_pos && write_pos - call_pos < 100 {
                    vulnerabilities.push(Vulnerability {
                        vuln_type: VulnerabilityType::ReentrancyAttack,
                        severity: 9,
                        location: format!("Call: 0x{:x}, Write: 0x{:x}", call_pos, write_pos),
                        description: "State change after external call detected".to_string(),
                        suggested_fix: "Move state changes before external calls".to_string(),
                        confidence: 0.75,
                    });
                    break; // Only report once per call
                }
            }
        }

        vulnerabilities
    }

    /// Control flow analysis
    fn analyze_control_flow(&self, bytecode: &[u8]) -> Vec<Vulnerability> {
        let mut vulnerabilities = Vec::new();
        let mut jump_targets = HashSet::new();
        let mut jump_destinations = HashSet::new();

        // Find all JUMP and JUMPI instructions and their targets
        let mut i = 0;
        while i < bytecode.len() {
            match bytecode[i] {
                0x56 => {
                    // JUMP
                    jump_targets.insert(i);
                }
                0x57 => {
                    // JUMPI
                    jump_targets.insert(i);
                }
                0x5b => {
                    // JUMPDEST
                    jump_destinations.insert(i);
                }
                0x60..=0x7f => {
                    // PUSH
                    let push_size = (bytecode[i] - 0x5f) as usize;
                    i += push_size;
                }
                _ => {}
            }
            i += 1;
        }

        // Check for unreachable code
        if jump_targets.len() > 20 && jump_destinations.len() < jump_targets.len() / 2 {
            vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::GasOptimization,
                severity: 3,
                location: "Control flow analysis".to_string(),
                description: "Complex control flow detected - may contain unreachable code".to_string(),
                suggested_fix: "Simplify logic and remove dead code".to_string(),
                confidence: 0.5,
            });
        }

        vulnerabilities
    }

    /// Access control analysis
    fn analyze_access_control(&self, bytecode: &[u8]) -> Vec<Vulnerability> {
        let mut vulnerabilities = Vec::new();
        let mut has_caller_check = false;
        let mut has_state_change = false;
        let mut has_selfdestruct = false;

        for &opcode in bytecode {
            match opcode {
                0x33 => has_caller_check = true, // CALLER
                0x55 => has_state_change = true,  // SSTORE
                0xff => has_selfdestruct = true,  // SELFDESTRUCT
                _ => {}
            }
        }

        // If contract has state changes but no caller checks
        if has_state_change && !has_caller_check {
            vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::UnauthorizedAccess,
                severity: 8,
                location: "Access control analysis".to_string(),
                description: "No access control detected for state-changing operations".to_string(),
                suggested_fix: "Implement role-based access control (RBAC)".to_string(),
                confidence: 0.7,
            });
        }

        // If contract has SELFDESTRUCT but no caller checks
        if has_selfdestruct && !has_caller_check {
            vulnerabilities.push(Vulnerability {
                vuln_type: VulnerabilityType::UnprotectedSelfDestruct,
                severity: 10,
                location: "SELFDESTRUCT found".to_string(),
                description: "SELFDESTRUCT without proper authorization".to_string(),
                suggested_fix: "Add owner-only modifier to selfdestruct function".to_string(),
                confidence: 0.9,
            });
        }

        vulnerabilities
    }

    /// Detect gas inefficiencies
    fn detect_gas_inefficiencies(&self, bytecode: &[u8]) -> Vec<String> {
        let mut tips = Vec::new();
        let mut sstore_count = 0;
        let mut sload_count = 0;
        let mut dup_count = 0;

        for &opcode in bytecode {
            match opcode {
                0x54 => sload_count += 1,       // SLOAD
                0x55 => sstore_count += 1,      // SSTORE
                0x80..=0x8f => dup_count += 1, // DUP
                _ => {}
            }
        }

        if sstore_count > 10 {
            tips.push(format!(
                "High SSTORE count ({}). Consider batching state updates",
                sstore_count
            ));
        }

        if sload_count > 20 {
            tips.push(format!(
                "Excessive SLOAD operations ({}). Cache storage in memory",
                sload_count
            ));
        }

        if dup_count > 50 {
            tips.push("Heavy stack manipulation detected. Refactor for clarity".to_string());
        }

        // Calculate estimated gas cost
        let estimated_cost: u64 = bytecode
            .iter()
            .map(|&op| *self.opcode_costs.get(&op).unwrap_or(&3))
            .sum();

        if estimated_cost > 1_000_000 {
            tips.push(format!(
                "High estimated gas cost: {}. Consider splitting into multiple transactions",
                estimated_cost
            ));
        }

        tips
    }

    /// Detect safe patterns
    fn detect_safe_patterns(&self, bytecode: &[u8]) -> Vec<String> {
        let mut patterns = Vec::new();

        // Check for overflow protection (ISZERO after ADD/MUL/SUB)
        let mut i = 0;
        while i + 2 < bytecode.len() {
            if (bytecode[i] == 0x01 || bytecode[i] == 0x02 || bytecode[i] == 0x03)
                && bytecode[i + 1] == 0x15
            {
                patterns.push("Overflow check detected".to_string());
                break;
            }
            i += 1;
        }

        // Check for reentrancy guard (SLOAD check before CALL)
        i = 0;
        while i + 3 < bytecode.len() {
            if bytecode[i] == 0x54 && bytecode[i + 1] == 0x15 && bytecode[i + 2] == 0x57 {
                patterns.push("Reentrancy guard pattern detected".to_string());
                break;
            }
            i += 1;
        }

        // Check for return value checks (ISZERO after CALL)
        i = 0;
        while i + 2 < bytecode.len() {
            if (bytecode[i] == 0xf1 || bytecode[i] == 0xf4) && bytecode[i + 1] == 0x15 {
                patterns.push("External call return value checked".to_string());
                break;
            }
            i += 1;
        }

        if patterns.is_empty() {
            patterns.push("No obvious security patterns detected".to_string());
        }

        patterns
    }

    /// Calculate contract complexity
    fn calculate_complexity(&self, bytecode: &[u8]) -> u32 {
        let mut complexity = 0u32;
        let mut jump_count = 0;
        let mut call_count = 0;

        for &opcode in bytecode {
            match opcode {
                0x56 | 0x57 => jump_count += 1,             // JUMP/JUMPI
                0xf1 | 0xf2 | 0xf4 | 0xfa => call_count += 1, // CALL variants
                _ => {}
            }
        }

        complexity += bytecode.len() as u32 / 10; // Base complexity
        complexity += jump_count * 5; // Jumps add complexity
        complexity += call_count * 10; // External calls add more

        complexity
    }

    /// Calculate overall security score
    fn calculate_security_score(&self, vulnerabilities: &[Vulnerability], complexity: u32) -> u8 {
        if vulnerabilities.is_empty() {
            return 100;
        }

        let total_severity: u32 = vulnerabilities
            .iter()
            .map(|v| v.severity as u32 * (v.confidence * 100.0) as u32 / 100)
            .sum();

        let complexity_penalty = (complexity / 20).min(20);
        let vulnerability_penalty = (total_severity * 2).min(80);

        (100u32
            .saturating_sub(vulnerability_penalty)
            .saturating_sub(complexity_penalty)) as u8
    }
}

impl Default for ContractAuditor {
    fn default() -> Self {
        Self::new()
    }
}
