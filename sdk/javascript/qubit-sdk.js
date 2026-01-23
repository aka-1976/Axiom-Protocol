/**
 * AXIOM Protocol JavaScript SDK
 * 
 * Provides a high-level interface for interacting with the Axiom blockchain:
 * - Wallet management (key generation, signing)
 * - Transaction creation and broadcasting
 * - Block and transaction queries
 * - ZK-SNARK proof generation for private transactions
 * - VDF verification
 * - Neural Guardian threat detection queries
 */

const crypto = require('crypto');
const fetch = require('node-fetch');


/**
 * Transaction class
 */
class Transaction {
    constructor(sender, recipient, amount, fee, nonce, timestamp) {
        this.sender = sender;
        this.recipient = recipient;
        this.amount = amount;
        this.fee = fee;
        this.nonce = nonce;
        this.timestamp = timestamp;
        this.signature = null;
        this.zkProof = null;
    }

    /**
     * Serialize transaction for signing
     */
    serialize() {
        return JSON.stringify({
            sender: this.sender,
            recipient: this.recipient,
            amount: this.amount,
            fee: this.fee,
            nonce: this.nonce,
            timestamp: this.timestamp
        }, Object.keys({sender:0, recipient:0, amount:0, fee:0, nonce:0, timestamp:0}).sort());
    }

    /**
     * Compute transaction hash (double SHA-256)
     */
    hash() {
        const firstHash = crypto.createHash('sha256').update(this.serialize()).digest();
        const secondHash = crypto.createHash('sha256').update(firstHash).digest('hex');
        return secondHash;
    }

    /**
     * Convert to plain object
     */
    toJSON() {
        return {
            sender: this.sender,
            recipient: this.recipient,
            amount: this.amount,
            fee: this.fee,
            nonce: this.nonce,
            timestamp: this.timestamp,
            signature: this.signature,
            zk_proof: this.zkProof
        };
    }
}


/**
 * Block class
 */
class Block {
    constructor(data) {
        this.index = data.index;
        this.timestamp = data.timestamp;
        this.transactions = data.transactions;
        this.previousHash = data.previous_hash || data.previousHash;
        this.merkleRoot = data.merkle_root || data.merkleRoot;
        this.nonce = data.nonce;
        this.difficulty = data.difficulty;
        this.vdfOutput = data.vdf_output || data.vdfOutput;
        this.vdfProof = data.vdf_proof || data.vdfProof;
        this.hash = data.hash;
    }
}


/**
 * Wallet class for key management and signing
 */
class Wallet {
    /**
     * Initialize wallet with private key or generate new one
     * @param {string|null} privateKey - 64-char hex private key (32 bytes), or null to generate
     */
    constructor(privateKey = null) {
        if (privateKey) {
            this.privateKey = privateKey;
        } else {
            this.privateKey = this._generatePrivateKey();
        }
        
        this.publicKey = this._derivePublicKey(this.privateKey);
        this.address = this._deriveAddress(this.publicKey);
    }

    /**
     * Generate random 256-bit private key
     */
    _generatePrivateKey() {
        return crypto.randomBytes(32).toString('hex');
    }

    /**
     * Derive public key from private key using Ed25519
     */
    _derivePublicKey(privateKey) {
        // Simplified: In production, use proper Ed25519 key derivation
        const data = Buffer.from(privateKey, 'hex');
        const pub = crypto.createHash('sha256')
            .update(Buffer.concat([data, Buffer.from('public')]))
            .digest('hex');
        return pub;
    }

    /**
     * Derive address from public key (SHA-256 hash)
     */
    _deriveAddress(publicKey) {
        return crypto.createHash('sha256')
            .update(Buffer.from(publicKey, 'hex'))
            .digest('hex');
    }

    /**
     * Sign a message with the wallet's private key
     * @param {string} message - Message to sign (typically serialized transaction)
     * @returns {string} 128-char hex signature (64 bytes)
     */
    sign(message) {
        // Simplified Ed25519 signature (production: use proper Ed25519)
        const msgHash = crypto.createHash('sha256').update(message).digest();
        const keyBytes = Buffer.from(this.privateKey, 'hex');
        const sigData = crypto.createHash('sha256')
            .update(Buffer.concat([keyBytes, msgHash]))
            .digest();
        // Pad to 64 bytes
        const signature = Buffer.concat([sigData, sigData]).toString('hex');
        return signature;
    }

    /**
     * Verify a signature
     * @param {string} message - Original message
     * @param {string} signature - 128-char hex signature
     * @param {string} publicKey - 64-char hex public key
     * @returns {boolean} True if signature is valid
     */
    static verify(message, signature, publicKey) {
        // Simplified verification (production: use proper Ed25519)
        const msgHash = crypto.createHash('sha256').update(message).digest();
        const expectedSig = crypto.createHash('sha256')
            .update(Buffer.concat([Buffer.from(publicKey, 'hex'), msgHash]))
            .digest();
        const actualSig = Buffer.from(signature, 'hex').slice(0, 32);
        return expectedSig.equals(actualSig);
    }
}


/**
 * Client for interacting with Axiom node RPC API
 */
class AxiomClient {
    /**
     * Initialize client
     * @param {string} nodeUrl - URL of Axiom node RPC endpoint
     */
    constructor(nodeUrl = 'http://localhost:8332') {
        this.nodeUrl = nodeUrl.replace(/\/$/, '');
    }

    /**
     * Make RPC call to node
     */
    async _rpcCall(method, params = {}) {
        const payload = {
            jsonrpc: '2.0',
            id: Date.now(),
            method: method,
            params: params
        };

        try {
            const response = await fetch(`${this.nodeUrl}/rpc`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json'
                },
                body: JSON.stringify(payload),
                timeout: 10000
            });

            if (!response.ok) {
                throw new Error(`HTTP error: ${response.status}`);
            }

            const result = await response.json();

            if (result.error) {
                throw new Error(`RPC error: ${JSON.stringify(result.error)}`);
            }

            return result.result || {};

        } catch (error) {
            throw new Error(`Network error: ${error.message}`);
        }
    }

    /**
     * Get balance for an address
     * @param {string} address - 64-char hex address
     * @returns {number} Balance in satoshis
     */
    async getBalance(address) {
        return await this._rpcCall('get_balance', { address });
    }

    /**
     * Get current nonce for an address
     * @param {string} address - 64-char hex address
     * @returns {number} Current nonce
     */
    async getNonce(address) {
        return await this._rpcCall('get_nonce', { address });
    }

    /**
     * Broadcast a signed transaction
     * @param {Transaction} tx - Signed transaction
     * @returns {string} Transaction hash
     */
    async broadcastTransaction(tx) {
        return await this._rpcCall('broadcast_transaction', tx.toJSON());
    }

    /**
     * Get transaction by hash
     * @param {string} txHash - 64-char hex transaction hash
     * @returns {object|null} Transaction data or null if not found
     */
    async getTransaction(txHash) {
        try {
            return await this._rpcCall('get_transaction', { hash: txHash });
        } catch {
            return null;
        }
    }

    /**
     * Get block by hash or index
     * @param {string|null} blockHash - 64-char hex block hash
     * @param {number|null} index - Block index
     * @returns {Block|null} Block object or null if not found
     */
    async getBlock(blockHash = null, index = null) {
        const params = {};
        if (blockHash) {
            params.hash = blockHash;
        } else if (index !== null) {
            params.index = index;
        } else {
            throw new Error('Must provide either blockHash or index');
        }

        try {
            const data = await this._rpcCall('get_block', params);
            return new Block(data);
        } catch {
            return null;
        }
    }

    /**
     * Get the latest block in the chain
     * @returns {Block} Latest block
     */
    async getLatestBlock() {
        const data = await this._rpcCall('get_latest_block');
        return new Block(data);
    }

    /**
     * Get blockchain info
     * @returns {object} Dict with height, difficulty, total_supply, etc.
     */
    async getChainInfo() {
        return await this._rpcCall('get_chain_info');
    }

    /**
     * Create and sign a transaction
     * @param {Wallet} wallet - Sender wallet
     * @param {string} recipient - Recipient address (64-char hex)
     * @param {number} amount - Amount in satoshis
     * @param {number} fee - Transaction fee (default 1000 sats = 0.00001 AXM)
     * @param {boolean} useZk - Whether to generate ZK-SNARK proof for privacy
     * @returns {Transaction} Signed transaction ready to broadcast
     */
    async createTransaction(wallet, recipient, amount, fee = 1000, useZk = false) {
        const nonce = await this.getNonce(wallet.address);
        
        const tx = new Transaction(
            wallet.address,
            recipient,
            amount,
            fee,
            nonce,
            Math.floor(Date.now() / 1000)
        );

        // Sign transaction
        tx.signature = wallet.sign(tx.serialize());

        // Generate ZK proof if requested
        if (useZk) {
            tx.zkProof = await this._generateZkProof(tx, wallet);
        }

        return tx;
    }

    /**
     * Generate ZK-SNARK proof for private transaction
     */
    async _generateZkProof(tx, wallet) {
        const proofData = await this._rpcCall('generate_zk_proof', {
            sender: tx.sender,
            amount: tx.amount,
            private_key: wallet.privateKey  // Sent over secure channel
        });
        return proofData.proof || '';
    }

    /**
     * Verify VDF proof
     * @param {string} vdfOutput - VDF output value
     * @param {string} vdfProof - Wesolowski proof
     * @param {string} inputData - VDF input
     * @param {number} timeParam - Time parameter
     * @returns {boolean} True if proof is valid
     */
    async verifyVdf(vdfOutput, vdfProof, inputData, timeParam) {
        const result = await this._rpcCall('verify_vdf', {
            output: vdfOutput,
            proof: vdfProof,
            input: inputData,
            time: timeParam
        });
        return result.valid || false;
    }

    /**
     * Query Neural Guardian for threat assessment of a peer
     * @param {string} peerId - Peer identifier
     * @returns {object} Dict with threat_type, confidence, action
     */
    async queryNeuralGuardian(peerId) {
        return await this._rpcCall('neural_guardian_query', { peer_id: peerId });
    }

    /**
     * Convenience method to create, sign, and broadcast a transaction
     * @param {Wallet} wallet - Sender wallet
     * @param {string} recipient - Recipient address
     * @param {number} amount - Amount in satoshis
     * @param {number} fee - Transaction fee
     * @param {boolean} useZk - Use ZK-SNARK for privacy
     * @returns {string} Transaction hash
     */
    async send(wallet, recipient, amount, fee = 1000, useZk = false) {
        const tx = await this.createTransaction(wallet, recipient, amount, fee, useZk);
        return await this.broadcastTransaction(tx);
    }
}


/**
 * Convert AXM to satoshis (1 AXM = 10^8 sats)
 */
function axmToSats(axm) {
    return Math.floor(axm * 100_000_000);
}


/**
 * Convert satoshis to AXM
 */
function satsToQbt(sats) {
    return sats / 100_000_000;
}


// Export classes and functions
module.exports = {
    Transaction,
    Block,
    Wallet,
    AxiomClient,
    axmToSats,
    satsToAxm
};


// Example usage
if (require.main === module) {
    (async () => {
        // Initialize client
        const client = new AxiomClient('http://localhost:8332');
        
        // Create or load wallet
        const wallet = new Wallet();  // Generate new wallet
        console.log(`Address: ${wallet.address}`);
        
        try {
            // Check balance
            const balance = await client.getBalance(wallet.address);
            console.log(`Balance: ${satsToAxm(balance)} AXM`);
            
            // Send transaction
            const recipient = 'a'.repeat(64);  // Example recipient address
            const amount = axmToSats(1.5);  // Send 1.5 AXM
            
            const txHash = await client.send(wallet, recipient, amount, 1000, true);
            console.log(`Transaction sent: ${txHash}`);
            
            // Get chain info
            const info = await client.getChainInfo();
            console.log(`Chain height: ${info.height}`);
            console.log(`Total supply: ${satsToAxm(info.total_supply || 0)} AXM`);
            
        } catch (error) {
            console.error(`Error: ${error.message}`);
        }
    })();
}
