# AXIOM Protocol Security Model (Draft)

## Threats Considered
- Double-spend attacks
- 51% attacks
- Sybil attacks
- Eclipse attacks
- DDoS attacks
- Long-range attacks
- Selfish mining

## Current Protections
- Basic transaction validation
- Heuristic-based peer monitoring
- No formal Sybil/eclipse/DDOS protection yet
- No VDF proof verification
- No trusted setup needed for ZK-STARKs (transparent)

## Security Assumptions
- Honest majority of nodes
- Secure cryptographic primitives (subject to review)
- No adversarial network conditions

## Gaps & TODO
- Implement VDF proof verification
- Add Sybil/eclipse/DDOS protection
- Document transparent setup for ZK-STARKs
- Formalize fork choice and reorg limits
- Add adversarial and fuzz testing

---
This model will be updated as security features are added.