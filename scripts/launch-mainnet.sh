#!/bin/bash
# AXIOM Protocol - Mainnet Launch Coordinator
# Orchestrates the complete mainnet launch process

set -e

echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║            AXIOM Protocol - Mainnet Launch Coordinator               ║"
echo "║                  Complete Deployment Workflow                         ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BLUE='\033[0;34m'
NC='\033[0m'

# Phase tracker
PHASE=1
TOTAL_PHASES=7

show_phase() {
    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════${NC}"
    echo -e "${YELLOW}Phase $PHASE/$TOTAL_PHASES: $1${NC}"
    echo -e "${BLUE}═══════════════════════════════════════════════════════════════════════${NC}"
    echo ""
    PHASE=$((PHASE + 1))
}

# Check prerequisites
show_phase "Prerequisites Check"

echo "Checking required files..."
required_files=(
    "generate-genesis.sh"
    "deploy-cloud-validators.sh"
    "target/release/axiom-node"
)

for file in "${required_files[@]}"; do
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓ $file${NC}"
    else
        echo -e "${RED}✗ $file not found${NC}"
        if [ "$file" == "target/release/axiom-node" ]; then
            echo "  Run: cargo build --release"
        fi
        exit 1
    fi
done

echo ""
read -p "Have you provisioned 5 cloud servers? (y/n): " servers_ready
if [ "$servers_ready" != "y" ] && [ "$servers_ready" != "Y" ]; then
    echo ""
    echo -e "${YELLOW}Please provision 5 cloud servers first:${NC}"
    echo ""
    echo "Recommended providers:"
    echo "  • AWS EC2 (t3.large, 8GB RAM, 50GB SSD)"
    echo "  • Google Cloud Compute (n1-standard-2)"
    echo "  • Azure Virtual Machines (Standard_B2s)"
    echo "  • DigitalOcean Droplets (8GB/4vCPU)"
    echo ""
    echo "Requirements per server:"
    echo "  • Ubuntu 22.04 LTS"
    echo "  • 8GB RAM minimum"
    echo "  • 50GB SSD minimum"
    echo "  • SSH access configured"
    echo "  • Public IP address"
    echo ""
    exit 0
fi

# Deploy cloud infrastructure
show_phase "Cloud Infrastructure Deployment"

echo "This will deploy validators to your cloud servers."
echo ""
read -p "Proceed with cloud deployment? (y/n): " deploy_confirm

if [ "$deploy_confirm" == "y" ] || [ "$deploy_confirm" == "Y" ]; then
    chmod +x deploy-cloud-validators.sh
    ./deploy-cloud-validators.sh
    
    if [ $? -ne 0 ]; then
        echo -e "${RED}✗ Cloud deployment failed${NC}"
        exit 1
    fi
else
    echo "Skipping cloud deployment. Make sure validators are deployed manually."
fi

# Verify deployment info exists
if [ ! -f "deployment-info.txt" ]; then
    echo -e "${RED}✗ deployment-info.txt not found${NC}"
    echo "Make sure cloud deployment completed successfully"
    exit 1
fi

# Extract validator addresses
show_phase "Collecting Validator Information"

echo "Extracting validator addresses from deployment..."
VALIDATOR_ADDRESSES=()
while IFS= read -r line; do
    if [[ $line == *"Address:"* ]]; then
        addr=$(echo "$line" | awk '{print $2}')
        VALIDATOR_ADDRESSES+=("$addr")
        echo -e "${GREEN}✓ Found validator: $addr${NC}"
    fi
done < deployment-info.txt

echo ""
echo -e "${GREEN}✓ Collected ${#VALIDATOR_ADDRESSES[@]} validator addresses${NC}"

if [ ${#VALIDATOR_ADDRESSES[@]} -lt 5 ]; then
    echo -e "${RED}✗ Need 5 validators, found ${#VALIDATOR_ADDRESSES[@]}${NC}"
    exit 1
fi

# Generate genesis block
show_phase "Genesis Block Generation"

echo "Generating genesis block with collected validators..."
echo ""

# Create genesis with validator addresses
chmod +x generate-genesis.sh

# Pass validator addresses to genesis script
{
    for addr in "${VALIDATOR_ADDRESSES[@]}"; do
        echo "$addr"
    done
    echo ""  # Empty line to finish input
} | ./generate-genesis.sh

if [ ! -f "genesis.json" ]; then
    echo -e "${RED}✗ Genesis block generation failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Genesis block created${NC}"

# Distribute genesis block
show_phase "Genesis Block Distribution"

echo "Distributing genesis.json to all validators..."
echo ""

if [ -f "deployment-info.txt" ]; then
    # Extract server info and distribute
    while IFS= read -r line; do
        if [[ $line == *"IP:"* ]]; then
            ip=$(echo "$line" | awk '{print $2}')
        elif [[ $line == *"User:"* ]]; then
            user=$(echo "$line" | awk '{print $2}')
            
            # Try to copy genesis
            echo "Copying to $user@$ip..."
            if scp -o StrictHostKeyChecking=no genesis.json "$user@$ip:~/Axiom-Protocol/" 2>/dev/null; then
                echo -e "${GREEN}✓ Copied to $ip${NC}"
            else
                echo -e "${YELLOW}⚠ Could not auto-copy to $ip. Please copy manually.${NC}"
            fi
        fi
    done < deployment-info.txt
fi

echo ""
echo -e "${GREEN}✓ Genesis distribution complete${NC}"

# Setup monitoring
show_phase "Monitoring Stack Deployment"

echo "Setting up centralized monitoring..."
echo ""

if [ -d "monitoring" ]; then
    # Update Prometheus config with real IPs
    if [ -f "deployment-info.txt" ]; then
        echo "Generating Prometheus targets..."
        
        cat > monitoring/prometheus/mainnet-targets.yml << EOF
# AXIOM Mainnet Prometheus Targets
# Auto-generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

- job_name: 'axiom-mainnet-validators'
  static_configs:
    - targets:
EOF
        
        while IFS= read -r line; do
            if [[ $line == *"IP:"* ]]; then
                ip=$(echo "$line" | awk '{print $2}')
                echo "      - '$ip:9100'" >> monitoring/prometheus/mainnet-targets.yml
            fi
        done < deployment-info.txt
        
        echo ""
        echo "      labels:" >> monitoring/prometheus/mainnet-targets.yml
        echo "        network: 'mainnet'" >> monitoring/prometheus/mainnet-targets.yml
        echo "        chain_id: '84000'" >> monitoring/prometheus/mainnet-targets.yml
        
        echo -e "${GREEN}✓ Prometheus targets configured${NC}"
    fi
    
    echo ""
    read -p "Start monitoring stack now? (y/n): " start_monitoring
    if [ "$start_monitoring" == "y" ] || [ "$start_monitoring" == "Y" ]; then
        cd monitoring
        docker-compose up -d
        cd ..
        echo -e "${GREEN}✓ Monitoring stack started${NC}"
        echo "  Grafana: http://localhost:3000 (admin/<set-your-password>)"
        echo "  Prometheus: http://localhost:9090"
    fi
else
    echo -e "${YELLOW}⚠ Monitoring directory not found. Skipping.${NC}"
fi

# Launch coordination
show_phase "Launch Coordination"

echo ""
echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║                   READY FOR MAINNET LAUNCH                            ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""

GENESIS_HASH=$(sha256sum genesis.json | awk '{print $1}')
GENESIS_TIME=$(grep '"timestamp"' genesis.json | awk -F'"' '{print $4}')

echo -e "${YELLOW}Network Information:${NC}"
echo "  Chain ID: 84000"
echo "  Network ID: 1"
echo "  Genesis Hash: $GENESIS_HASH"
echo "  Genesis Time: $GENESIS_TIME"
echo "  Validators: ${#VALIDATOR_ADDRESSES[@]}"
echo ""

echo -e "${YELLOW}Validator Servers:${NC}"
grep -E "Name:|IP:|RPC:" deployment-info.txt | paste - - - | while read line; do
    echo "  $line"
done
echo ""

echo -e "${YELLOW}Launch Commands (SSH to each server):${NC}"
echo ""
echo "  ssh user@server-ip"
echo "  cd ~/Axiom-Protocol"
echo "  nohup ./target/release/axiom-node --config axiom-validator.toml > validator.log 2>&1 &"
echo ""

echo -e "${YELLOW}Verify Launch:${NC}"
echo "  Watch logs: tail -f ~/Axiom-Protocol/validator.log"
echo "  Check status: curl http://server-ip:RPC_PORT"
echo "  Monitor peers: grep 'peers' ~/Axiom-Protocol/validator.log"
echo ""

echo -e "${YELLOW}Monitoring:${NC}"
echo "  Grafana Dashboard: http://localhost:3000"
echo "  Validator metrics in real-time"
echo ""

read -p "Launch validators now? (y/n): " launch_now

if [ "$launch_now" == "y" ] || [ "$launch_now" == "Y" ]; then
    echo ""
    echo -e "${YELLOW}Launching validators...${NC}"
    
    # Extract SSH info and launch
    declare -A server_info
    current_validator=0
    
    while IFS= read -r line; do
        if [[ $line == *"IP:"* ]]; then
            ip=$(echo "$line" | awk '{print $2}')
        elif [[ $line == *"User:"* ]]; then
            user=$(echo "$line" | awk '{print $2}')
            current_validator=$((current_validator + 1))
            
            echo "Starting validator $current_validator on $ip..."
            ssh -o StrictHostKeyChecking=no "$user@$ip" << 'ENDSSH' &
cd ~/Axiom-Protocol
nohup ./target/release/axiom-node --config axiom-validator.toml > validator.log 2>&1 &
echo "Validator started. Check logs: tail -f validator.log"
ENDSSH
            
            sleep 2
        fi
    done < deployment-info.txt
    
    echo ""
    echo -e "${GREEN}✓ Launch commands sent to all validators${NC}"
    echo ""
    echo "Waiting 30 seconds for startup..."
    sleep 30
    
    # Check validator status
    echo ""
    echo "Checking validator status..."
    while IFS= read -r line; do
        if [[ $line == *"RPC:"* ]]; then
            rpc_url=$(echo "$line" | awk '{print $2}')
            if curl -s --max-time 5 "$rpc_url" >/dev/null 2>&1; then
                echo -e "${GREEN}✓ $rpc_url - ONLINE${NC}"
            else
                echo -e "${YELLOW}⚠ $rpc_url - Starting...${NC}"
            fi
        fi
    done < deployment-info.txt
fi

echo ""
echo "╔═══════════════════════════════════════════════════════════════════════╗"
echo "║              🎉 AXIOM MAINNET LAUNCH COMPLETE 🎉                      ║"
echo "╚═══════════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}✨ AXIOM Protocol is now live on mainnet!${NC}"
echo ""
echo -e "${YELLOW}Post-Launch Tasks:${NC}"
echo "  • Monitor validator logs for first block production"
echo "  • Verify all 5 validators are peering"
echo "  • Check Grafana dashboards"
echo "  • Announce mainnet launch to community"
echo "  • Enable public RPC endpoints"
echo "  • Launch block explorer"
echo ""
echo -e "${YELLOW}Support:${NC}"
echo "  • Documentation: https://docs.axiom.network"
echo "  • Discord: https://discord.gg/axiom-network"
echo "  • Email: support@axiom.network"
echo ""
echo -e "${GREEN}Genesis Hash: $GENESIS_HASH${NC}"
echo ""
