# AXIOM Protocol Economics & Tokenomics

## Total Supply
- **Fixed supply**: 124,000,000 AXM (124 million)
- **Initial reward**: 50 AXM per block
- **Halving interval**: Every 1,240,000 blocks (~70.7 years per era)
- **Block time**: 30 minutes (1800 seconds)
- No inflation or deflation mechanisms - mathematically perfect scarcity

## Block Rewards
- **Binary halving schedule**: 50 → 25 → 12.5 → 6.25 → ...
- **Generation duration**: ~70.7 years per halving era
- **Total eras**: 64 halvings until reward reaches 0
- **Supply curve**: 100% mined after ~33 eras (around year 850)
- No priority fee mechanism (EIP-1559 style, planned)
- Minimum fee enforced: 0.01 AXM (1,000,000 units)

## Network Phases
1. **Pillar Era (0-5 years)**: Foundation building, early adopters, ~2,117 blocks
2. **Infrastructure Era (5-10 years)**: Scaling, ecosystem growth, ~4,233 blocks
3. **Sovereign Era (10-20 years)**: Maturity, global adoption, ~6,350 blocks
4. **Maturity Era (20+ years)**: Long-term stability, established network

## Difficulty Adjustment
- **Algorithm**: LWMA (Linear Weighted Moving Average)
- **Window**: 60 blocks (~30 hours)
- **Max adjustment**: 3x per adjustment period
- **Target**: 1800 seconds (30 minutes) per block

## Transaction Fees
- Dynamic fee market logic (fees can be adjusted based on network demand)
- Minimum fee required for all transactions (prevents spam)

## Distribution
- Fair initial distribution: No premine, no central treasury, no burn mechanism
- All coins are mined via consensus, rewards are distributed to miners
- Genesis block: Block 0 with 0 supply (all mined through PoW)

## Economic Security
- Minimum fee and dynamic fee market help prevent spam and incentivize miners
- LWMA difficulty prevents flash mining attacks
- 30-minute blocks balance security and user experience
- Anti-centralization: Mining pool detection and reward capping implemented (max 10 consecutive blocks per address)

---
This document will be updated as economic mechanisms are designed. See economics.rs and chain.rs for minimum fee, reward logic, and anti-centralization mechanisms.