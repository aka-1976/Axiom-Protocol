# Security Policy

AXIOM Protocol is a decentralized, cryptography-driven system.
Security is a top priority, and responsible disclosure is strongly encouraged.

---

## 🔒 Supported Versions

Only the latest stable release on the `main` branch is actively supported
with security updates.

| Version / Branch | Supported |
|------------------|-----------|
| `main`           | ✅ Yes    |
| Older commits    | ❌ No     |
| Forks            | ❌ No     |

Running outdated versions may expose you to known or unknown vulnerabilities.

---

## 🚨 Reporting a Vulnerability

If you discover a security issue, **DO NOT** open a public GitHub issue.

### 📩 Report Privately
Please report vulnerabilities responsibly by contacting:

**Email:** `security@axiomprotocol.org`  
*(replace with your real email if different)*

If email is unavailable, you may:
- Open a **private GitHub Security Advisory**
- Or contact the maintainer directly via GitHub

---

## 🕒 Response Timeline

We aim to follow this disclosure timeline:

- **Acknowledgement:** within 48 hours  
- **Initial assessment:** within 5 days  
- **Fix or mitigation:** as soon as possible depending on severity  
- **Public disclosure:** after a fix is released (if applicable)

---

## 🛡 Scope of Security

The following are considered **in scope**:
- Consensus logic
- Cryptographic primitives
- Wallet & key management
- Networking (libp2p)
- Transaction validation
- Supply & issuance logic

The following are **out of scope**:
- Denial of service via spam without protocol exploit
- Social engineering attacks
- Issues in third-party dependencies unless exploitable through Axiom

---

## 🔍 Security Audits

### Current Audit Status (February 2026)

**Tool Used:** `cargo audit` (Rust security vulnerability scanner)

**Last Audit Run:** January 20, 2025

**Summary:**
- **Total Vulnerabilities:** 1 (down from 2 after dependency updates)
- **Warnings:** 7 (unmaintained/unsound crates, mostly in transitive dependencies)
- **Critical Issues:** 0

#### Active Vulnerabilities:
1. **tracing-subscriber** (transitive via ark-relations)
   - **Severity:** Medium
   - **Advisory:** RUSTSEC-2025-0055
   - **Description:** Logging user input may result in poisoning logs with ANSI escape sequences
   - **Status:** Upstream issue in arkworks/ark-relations. Patch attempted but blocked by dependency constraints.
   - **Mitigation:** AXIOM Protocol does not log untrusted user input in a way that would allow ANSI escape sequence injection. Risk assessed as low for current usage patterns.
   - **Tracking:** Monitoring for upstream fix in arkworks ecosystem

#### Resolved Vulnerabilities:
- **ring** (RUSTSEC-2024-0336): Resolved by updating libp2p to 0.56.

#### Warnings (Non-Critical):
- Several crates flagged as unmaintained or unsound, but not directly exploitable.
- Full list available by running `cargo audit` locally.

**Audit Command:** `cargo audit`

We recommend running `cargo audit` before building or deploying to check for the latest advisories.

---

## 🧠 Responsible Disclosure

We kindly ask reporters to:
- Avoid exploiting vulnerabilities beyond proof-of-concept
- Not disclose issues publicly before a fix
- Provide clear reproduction steps if possible

We appreciate and respect all responsible security researchers.

---

## ⚠️ Disclaimer

AXIOM Protocol is a running upcoming crypto project run and test for urself
