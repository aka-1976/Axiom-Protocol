#!/usr/bin/env python3
"""
AXIOM Protocol Application Example
A simple Python application that interacts with the Axiom blockchain
"""

import subprocess
import json
import time
from typing import Optional, Dict

class AxiomApp:
    def __init__(self, wallet_path: str = "./target/release/axiom-wallet"):
        self.wallet_path = wallet_path

    def get_wallet_address(self) -> Optional[str]:
        """Get the current wallet address"""
        try:
            result = subprocess.run([self.wallet_path, "export"],
                                  capture_output=True, text=True, check=True)
            return result.stdout.strip()
        except subprocess.CalledProcessError:
            return None

    def get_balance(self) -> float:
        """Get current wallet balance"""
        try:
            result = subprocess.run([self.wallet_path, "balance"],
                                  capture_output=True, text=True, check=True)
            # Parse balance from output like "Balance: 100.00000000 AXM"
            output = result.stdout.strip()
            if "Balance:" in output:
                balance_str = output.split("Balance:")[1].split("AXM")[0].strip()
                return float(balance_str)
            return 0.0
        except (subprocess.CalledProcessError, ValueError):
            return 0.0

    def send_transaction(self, to_address: str, amount: float, fee: float = 1.0) -> bool:
        """Send AXM to another address"""
        try:
            result = subprocess.run([
                self.wallet_path, "send", to_address, str(amount), str(fee)
            ], capture_output=True, text=True, check=True)
            return "Transaction created" in result.stdout
        except subprocess.CalledProcessError:
            return False

    def get_network_status(self) -> Dict:
        """Get network status from running node (placeholder)"""
        # In a real app, this would query the node's API
        return {
            "status": "demo",
            "peers": 2,
            "height": 1,
            "message": "Network monitoring active"
        }

def main():
    print("🚀 AXIOM Protocol Application Demo")
    print("=" * 40)

    app = AxiomApp()

    # Get wallet info
    address = app.get_wallet_address()
    if address:
        print(f"📧 Wallet Address: {address}")
    else:
        print("❌ Could not get wallet address")
        return

    # Get balance
    balance = app.get_balance()
    print(f"💰 Current Balance: {balance} AXM")

    # Get network status
    network = app.get_network_status()
    print(f"🌐 Network Status: {network['message']}")
    print(f"📊 Connected Peers: {network['peers']}")
    print(f"⛓️  Block Height: {network['height']}")

    # Demo transaction (won't work without funds)
    print("\n💸 Transaction Demo:")
    print("   Note: This would create a pending transaction")
    print("   In real usage: app.send_transaction(recipient_address, 10.0, 1.0)")
    # Uncomment to actually send: app.send_transaction(address, 10.0, 1.0)

    print("\n✅ AXIOM Protocol Application Ready!")
    print("💡 Next steps: Integrate with web APIs, build wallets, create dApps!")

if __name__ == "__main__":
    main()