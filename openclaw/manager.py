#!/usr/bin/env python3
"""
OpenClaw Integration Manager for Axiom Protocol
Orchestrates all three components: Ceremony Master, Node Health Monitor, Agent Internet
"""

import asyncio
import json
import sys
from typing import Dict, Optional

from ceremony_master import CeremonyMaster, CEREMONY_MASTER_SKILL
from node_health_monitor import NodeHealthMonitor, NODE_HEALTH_MONITOR_SKILL
from agent_internet import (
    AgentInternetIntegration,
    create_axiom_agents,
    AGENT_INTERNET_INTEGRATION,
)


class AxiomOpenClawManager:
    """Main orchestrator for Axiom OpenClaw integration"""

    def __init__(self):
        self.ceremony_master: Optional[CeremonyMaster] = None
        self.health_monitor: Optional[NodeHealthMonitor] = None
        self.agent_internet = AgentInternetIntegration()
        self.config = {}

    def load_config(self, config_path: str) -> None:
        """Load configuration from JSON file"""
        try:
            with open(config_path, "r") as f:
                self.config = json.load(f)
            print(f"✅ Configuration loaded from {config_path}")
        except FileNotFoundError:
            print(f"❌ Config file not found: {config_path}")
            self._create_default_config(config_path)

    def _create_default_config(self, config_path: str) -> None:
        """Create default configuration"""
        default_config = {
            "ceremony": {
                "enabled": True,
                "miners": [
                    {
                        "id": "miner_1",
                        "name": "Miner Alpha",
                        "contact": "miner1@axiom.network",
                        "endpoint": "http://0.0.0.0:8001",
                    },
                    {
                        "id": "miner_2",
                        "name": "Miner Beta",
                        "contact": "miner2@axiom.network",
                        "endpoint": "http://0.0.0.0:8002",
                    },
                    {
                        "id": "miner_3",
                        "name": "Miner Gamma",
                        "contact": "miner3@axiom.network",
                        "endpoint": "http://0.0.0.0:8003",
                    },
                ],
            },
            "monitoring": {
                "enabled": True,
                "node_id": "axiom_node_main",
                "check_interval_seconds": 30,
            },
            "agent_internet": {
                "enabled": True,
                "platforms": ["moltbook", "eliza", "autonolas"],
                "moltbook": {
                    "base_url": "https://axiom-ceremony.moltbook.ai",
                    "enabled": True,
                },
                "eliza": {
                    "base_url": "http://localhost:3000",
                    "enabled": True,
                },
                "autonolas": {
                    "reward_token": "OLAS",
                    "staking_amount": 5000,
                    "enabled": True,
                },
            },
        }

        with open(config_path, "w") as f:
            json.dump(default_config, f, indent=2)

        print(f"📝 Default config created: {config_path}")
        self.config = default_config

    async def initialize_ceremony_master(self) -> None:
        """Initialize Ceremony Master component"""
        if not self.config.get("ceremony", {}).get("enabled", True):
            print("⏭️  Ceremony Master disabled in config")
            return

        miners = self.config.get("ceremony", {}).get("miners", [])

        if not miners:
            print("❌ No miners configured for Ceremony Master")
            return

        self.ceremony_master = CeremonyMaster(miners)
        print("✅ Ceremony Master initialized")
        print(f"   Miners: {len(miners)}")
        for miner in miners:
            print(f"   - {miner['name']}: {miner['contact']}")

    async def initialize_health_monitor(self) -> None:
        """Initialize Node Health Monitor component"""
        if not self.config.get("monitoring", {}).get("enabled", True):
            print("⏭️  Node Health Monitor disabled in config")
            return

        monitoring_config = self.config.get("monitoring", {})
        node_id = monitoring_config.get("node_id", "axiom_node_main")
        check_interval = monitoring_config.get("check_interval_seconds", 30)

        self.health_monitor = NodeHealthMonitor(node_id, check_interval)
        print("✅ Node Health Monitor initialized")
        print(f"   Node ID: {node_id}")
        print(f"   Check Interval: {check_interval}s")

    async def initialize_agent_internet(self) -> None:
        """Initialize Agent Internet deployment"""
        if not self.config.get("agent_internet", {}).get("enabled", True):
            print("⏭️  Agent Internet disabled in config")
            return

        print("✅ Agent Internet initialized")
        agents = create_axiom_agents()
        print(f"   Platforms: Moltbook, Eliza, Autonolas")

    async def start_ceremony(self, ceremony_id: Optional[str] = None) -> Dict:
        """Start ZK ceremony automation"""
        if not self.ceremony_master:
            return {"error": "Ceremony Master not initialized"}

        return await self.ceremony_master.start_ceremony()

    async def start_monitoring(self) -> None:
        """Start continuous health monitoring"""
        if not self.health_monitor:
            print("❌ Health Monitor not initialized")
            return

        await self.health_monitor.start_monitoring()

    async def stop_monitoring(self) -> None:
        """Stop health monitoring"""
        if self.health_monitor:
            await self.health_monitor.stop_monitoring()

    def show_status(self) -> Dict:
        """Display current system status"""
        status = {
            "timestamp": asyncio.get_event_loop().time(),
            "ceremony_master": {
                "initialized": self.ceremony_master is not None,
                "status": None,
            },
            "health_monitor": {
                "initialized": self.health_monitor is not None,
                "status": None,
            },
            "agent_internet": {
                "initialized": True,
                "agents": len(self.agent_internet.agents),
                "deployments": len(self.agent_internet.deployments),
            },
        }

        if self.ceremony_master:
            status["ceremony_master"]["status"] = (
                asyncio.run(self.ceremony_master.get_status())
                if asyncio.iscoroutinefunction(self.ceremony_master.get_status)
                else "active"
            )

        return status

    async def interactive_console(self) -> None:
        """Interactive command console"""
        print("\n" + "=" * 60)
        print("🔧 Axiom OpenClaw Management Console")
        print("=" * 60)
        print("\nAvailable commands:")
        print("  start-ceremony    - Start ZK ceremony automation")
        print("  monitor-status    - Show current status")
        print("  ceremony-history  - Show ceremony handoff history")
        print("  health-report     - Show node health report")
        print("  export-config     - Export deployment config")
        print("  quit              - Exit console")
        print("=" * 60 + "\n")

        while True:
            try:
                cmd = input("axiom> ").strip().lower()

                if cmd == "quit":
                    print("👋 Goodbye!")
                    break

                elif cmd == "start-ceremony":
                    if self.ceremony_master:
                        result = await self.start_ceremony()
                        print(f"\n✅ Ceremony started: {json.dumps(result, indent=2)}\n")
                    else:
                        print("❌ Ceremony Master not initialized\n")

                elif cmd == "monitor-status":
                    status = self.show_status()
                    print(f"\n📊 Status: {json.dumps(status, indent=2)}\n")

                elif cmd == "ceremony-history":
                    if self.ceremony_master:
                        history = await self.ceremony_master.get_handoff_history()
                        print(
                            f"\n📜 Handoff History:\n{json.dumps(history, indent=2)}\n"
                        )
                    else:
                        print("❌ Ceremony Master not initialized\n")

                elif cmd == "health-report":
                    if self.health_monitor:
                        report = await self.health_monitor.get_peer_report()
                        print(f"\n🏥 Health Report:\n{json.dumps(report, indent=2)}\n")
                    else:
                        print("❌ Health Monitor not initialized\n")

                elif cmd == "export-config":
                    agents = self.agent_internet.list_agents()
                    print(f"\n📤 Agents ({len(agents)}):\n{json.dumps(agents, indent=2)}\n")

                else:
                    print("❓ Unknown command. Type 'quit' to exit.\n")

            except KeyboardInterrupt:
                print("\n👋 Interrupted!")
                break
            except Exception as e:
                print(f"❌ Error: {e}\n")


async def main():
    """Main entry point"""
    manager = AxiomOpenClawManager()

    # Load or create config
    manager.load_config("axiom_openclaw_config.json")

    # Initialize all components
    print("\n🚀 Initializing Axiom OpenClaw Components...\n")

    await manager.initialize_ceremony_master()
    await manager.initialize_health_monitor()
    await manager.initialize_agent_internet()

    print("\n✅ All components initialized!\n")

    # Start interactive console
    await manager.interactive_console()


if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("\n🛑 Shutdown requested")
        sys.exit(0)
