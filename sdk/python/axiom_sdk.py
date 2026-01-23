"""
AXIOM Protocol Python SDK

Provides a high-level interface for interacting with the Axiom blockchain:
- Wallet management (key generation, signing)
- Transaction creation and broadcasting
- Block and transaction queries
- ZK-SNARK proof generation for private transactions
- VDF verification
- Neural Guardian threat detection queries
"""

import hashlib
import json
import time
from typing import Optional, Dict, List, Tuple
import requests
from dataclasses import dataclass, asdict


@dataclass
class Transaction:
    """Axiom transaction structure"""
    sender: str  # 64-char hex address (32 bytes)
    recipient: str  # 64-char hex address
    amount: int  # Amount in satoshis (1 AXM = 10^8 sats)
    fee: int  # Transaction fee
    nonce: int  # Sender nonce to prevent replay attacks
    timestamp: int  # Unix timestamp
    signature: Optional[str] = None  # 128-char hex signature
    zk_proof: Optional[str] = None  # Optional ZK-SNARK proof for privacy

    def to_dict(self) -> dict:
        return asdict(self)

    def serialize(self) -> str:
        """Serialize transaction for signing"""
        return json.dumps({
            'sender': self.sender,
            'recipient': self.recipient,
            'amount': self.amount,
            'fee': self.fee,
            'nonce': self.nonce,
            'timestamp': self.timestamp
        }, sort_keys=True)

    def hash(self) -> str:
        """Compute transaction hash (double SHA-256)"""
        first_hash = hashlib.sha256(self.serialize().encode()).digest()
        second_hash = hashlib.sha256(first_hash).hexdigest()
        return second_hash


@dataclass
class Block:
    """Axiom block structure"""
    index: int
    timestamp: int
    transactions: List[Dict]
    previous_hash: str
    merkle_root: str
    nonce: int
    difficulty: int
    vdf_output: Optional[str] = None
    vdf_proof: Optional[str] = None
    hash: Optional[str] = None

    @classmethod
    def from_dict(cls, data: dict) -> 'Block':
        return cls(**data)


class Wallet:
    """Axiom wallet for key management and signing"""
    
    def __init__(self, private_key: Optional[str] = None):
        """
        Initialize wallet with private key or generate new one
        
        Args:
            private_key: 64-char hex private key (32 bytes), or None to generate
        """
        if private_key:
            self.private_key = private_key
        else:
            self.private_key = self._generate_private_key()
        
        self.public_key = self._derive_public_key(self.private_key)
        self.address = self._derive_address(self.public_key)
    
    @staticmethod
    def _generate_private_key() -> str:
        """Generate random 256-bit private key"""
        import secrets
        return secrets.token_hex(32)
    
    @staticmethod
    def _derive_public_key(private_key: str) -> str:
        """Derive public key from private key using Ed25519"""
        # Simplified: In production, use proper Ed25519 key derivation
        data = bytes.fromhex(private_key)
        pub = hashlib.sha256(data + b"public").hexdigest()
        return pub
    
    @staticmethod
    def _derive_address(public_key: str) -> str:
        """Derive address from public key (SHA-256 hash)"""
        return hashlib.sha256(bytes.fromhex(public_key)).hexdigest()
    
    def sign(self, message: str) -> str:
        """
        Sign a message with the wallet's private key
        
        Args:
            message: Message to sign (typically serialized transaction)
            
        Returns:
            128-char hex signature (64 bytes)
        """
        # Simplified Ed25519 signature (production: use proper Ed25519)
        msg_hash = hashlib.sha256(message.encode()).digest()
        key_bytes = bytes.fromhex(self.private_key)
        sig_data = hashlib.sha256(key_bytes + msg_hash).digest()
        # Pad to 64 bytes
        signature = (sig_data + sig_data).hex()
        return signature
    
    @staticmethod
    def verify(message: str, signature: str, public_key: str) -> bool:
        """
        Verify a signature
        
        Args:
            message: Original message
            signature: 128-char hex signature
            public_key: 64-char hex public key
            
        Returns:
            True if signature is valid
        """
        # Simplified verification (production: use proper Ed25519)
        msg_hash = hashlib.sha256(message.encode()).digest()
        expected_sig = hashlib.sha256(bytes.fromhex(public_key) + msg_hash).digest()
        actual_sig = bytes.fromhex(signature)[:32]
        return expected_sig == actual_sig


class AxiomClient:
    """Client for interacting with Axiom node RPC API"""
    
    def __init__(self, node_url: str = "http://localhost:8332"):
        """
        Initialize client
        
        Args:
            node_url: URL of Axiom node RPC endpoint
        """
        self.node_url = node_url.rstrip('/')
        self.session = requests.Session()
    
    def _rpc_call(self, method: str, params: dict = None) -> dict:
        """Make RPC call to node"""
        payload = {
            "jsonrpc": "2.0",
            "id": int(time.time() * 1000),
            "method": method,
            "params": params or {}
        }
        
        try:
            response = self.session.post(f"{self.node_url}/rpc", json=payload, timeout=10)
            response.raise_for_status()
            result = response.json()
            
            if "error" in result and result["error"]:
                raise Exception(f"RPC error: {result['error']}")
            
            return result.get("result", {})
        
        except requests.exceptions.RequestException as e:
            raise Exception(f"Network error: {e}")
    
    def get_balance(self, address: str) -> int:
        """
        Get balance for an address
        
        Args:
            address: 64-char hex address
            
        Returns:
            Balance in satoshis
        """
        return self._rpc_call("get_balance", {"address": address})
    
    def get_nonce(self, address: str) -> int:
        """
        Get current nonce for an address
        
        Args:
            address: 64-char hex address
            
        Returns:
            Current nonce
        """
        return self._rpc_call("get_nonce", {"address": address})
    
    def broadcast_transaction(self, tx: Transaction) -> str:
        """
        Broadcast a signed transaction
        
        Args:
            tx: Signed transaction
            
        Returns:
            Transaction hash
        """
        return self._rpc_call("broadcast_transaction", tx.to_dict())
    
    def get_transaction(self, tx_hash: str) -> Optional[Dict]:
        """
        Get transaction by hash
        
        Args:
            tx_hash: 64-char hex transaction hash
            
        Returns:
            Transaction data or None if not found
        """
        try:
            return self._rpc_call("get_transaction", {"hash": tx_hash})
        except:
            return None
    
    def get_block(self, block_hash: Optional[str] = None, index: Optional[int] = None) -> Optional[Block]:
        """
        Get block by hash or index
        
        Args:
            block_hash: 64-char hex block hash
            index: Block index
            
        Returns:
            Block object or None if not found
        """
        params = {}
        if block_hash:
            params["hash"] = block_hash
        elif index is not None:
            params["index"] = index
        else:
            raise ValueError("Must provide either block_hash or index")
        
        try:
            data = self._rpc_call("get_block", params)
            return Block.from_dict(data)
        except:
            return None
    
    def get_latest_block(self) -> Block:
        """Get the latest block in the chain"""
        data = self._rpc_call("get_latest_block")
        return Block.from_dict(data)
    
    def get_chain_info(self) -> Dict:
        """
        Get blockchain info
        
        Returns:
            Dict with height, difficulty, total_supply, etc.
        """
        return self._rpc_call("get_chain_info")
    
    def create_transaction(self, wallet: Wallet, recipient: str, amount: int, 
                          fee: int = 1000, use_zk: bool = False) -> Transaction:
        """
        Create and sign a transaction
        
        Args:
            wallet: Sender wallet
            recipient: Recipient address (64-char hex)
            amount: Amount in satoshis
            fee: Transaction fee (default 1000 sats = 0.00001 AXM)
            use_zk: Whether to generate ZK-SNARK proof for privacy
            
        Returns:
            Signed transaction ready to broadcast
        """
        nonce = self.get_nonce(wallet.address)
        
        tx = Transaction(
            sender=wallet.address,
            recipient=recipient,
            amount=amount,
            fee=fee,
            nonce=nonce,
            timestamp=int(time.time())
        )
        
        # Sign transaction
        tx.signature = wallet.sign(tx.serialize())
        
        # Generate ZK proof if requested
        if use_zk:
            tx.zk_proof = self._generate_zk_proof(tx, wallet)
        
        return tx
    
    def _generate_zk_proof(self, tx: Transaction, wallet: Wallet) -> str:
        """
        Generate ZK-SNARK proof for private transaction
        
        This proves the sender has sufficient balance without revealing
        the exact balance or transaction amount
        """
        # Call ZK prover service
        proof_data = self._rpc_call("generate_zk_proof", {
            "sender": tx.sender,
            "amount": tx.amount,
            "private_key": wallet.private_key  # Sent over secure channel
        })
        return proof_data.get("proof", "")
    
    def verify_vdf(self, vdf_output: str, vdf_proof: str, input_data: str, time_param: int) -> bool:
        """
        Verify VDF proof
        
        Args:
            vdf_output: VDF output value
            vdf_proof: Wesolowski proof
            input_data: VDF input
            time_param: Time parameter
            
        Returns:
            True if proof is valid
        """
        result = self._rpc_call("verify_vdf", {
            "output": vdf_output,
            "proof": vdf_proof,
            "input": input_data,
            "time": time_param
        })
        return result.get("valid", False)
    
    def query_neural_guardian(self, peer_id: str) -> Dict:
        """
        Query Neural Guardian for threat assessment of a peer
        
        Args:
            peer_id: Peer identifier
            
        Returns:
            Dict with threat_type, confidence, action
        """
        return self._rpc_call("neural_guardian_query", {"peer_id": peer_id})
    
    def send(self, wallet: Wallet, recipient: str, amount: int, 
            fee: int = 1000, use_zk: bool = False) -> str:
        """
        Convenience method to create, sign, and broadcast a transaction
        
        Args:
            wallet: Sender wallet
            recipient: Recipient address
            amount: Amount in satoshis
            fee: Transaction fee
            use_zk: Use ZK-SNARK for privacy
            
        Returns:
            Transaction hash
        """
        tx = self.create_transaction(wallet, recipient, amount, fee, use_zk)
        return self.broadcast_transaction(tx)


def axm_to_sats(axm: float) -> int:
    """Convert AXM to satoshis (1 AXM = 10^8 sats)"""
    return int(axm * 100_000_000)


def sats_to_axm(sats: int) -> float:
    """Convert satoshis to AXM"""
    return sats / 100_000_000


# Example usage
if __name__ == "__main__":
    # Initialize client
    client = AxiomClient("http://localhost:8332")
    
    # Create or load wallet
    wallet = Wallet()  # Generate new wallet
    print(f"Address: {wallet.address}")
    
    # Check balance
    balance = client.get_balance(wallet.address)
    print(f"Balance: {sats_to_axm(balance)} AXM")
    
    # Send transaction
    recipient = "a" * 64  # Example recipient address
    amount = axm_to_sats(1.5)  # Send 1.5 AXM
    
    try:
        tx_hash = client.send(wallet, recipient, amount, use_zk=True)
        print(f"Transaction sent: {tx_hash}")
    except Exception as e:
        print(f"Error: {e}")
    
    # Get chain info
    info = client.get_chain_info()
    print(f"Chain height: {info.get('height')}")
    print(f"Total supply: {sats_to_axm(info.get('total_supply', 0))} AXM")
