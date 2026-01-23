const hre = require("hardhat");

async function main() {
  const networkName = hre.network.name;
  console.log(`\n🚀 Deploying Axiom Bridge to ${networkName}...`);
  
  // Get deployer account
  const [deployer] = await hre.ethers.getSigners();
  console.log("Deploying with account:", deployer.address);
  console.log("Account balance:", (await deployer.getBalance()).toString());

  // Deploy Wrapped Axiom Token
  console.log("\n📦 Deploying Wrapped Axiom (wAXM)...");
  const WrappedAxiom = await hre.ethers.getContractFactory("WrappedAxiom");
  const wrappedToken = await WrappedAxiom.deploy();
  await wrappedToken.deployed();
  console.log("✅ wAXM deployed to:", wrappedToken.address);

  // Deploy Bridge Contract
  console.log("\n🌉 Deploying Bridge Contract...");
  const Bridge = await hre.ethers.getContractFactory("AxiomBridge");
  const bridge = await Bridge.deploy(wrappedToken.address);
  await bridge.deployed();
  console.log("✅ Bridge deployed to:", bridge.address);

  // Configure wrapped token
  console.log("\n⚙️  Configuring contracts...");
  const tx = await wrappedToken.setBridge(bridge.address);
  await tx.wait();
  console.log("✅ Bridge address set on wAXM token");

  // Display summary
  console.log("\n" + "=".repeat(60));
  console.log("📋 DEPLOYMENT SUMMARY");
  console.log("=".repeat(60));
  console.log("Network:", networkName);
  console.log("Deployer:", deployer.address);
  console.log("wAXM Token:", wrappedToken.address);
  console.log("Bridge Contract:", bridge.address);
  console.log("=".repeat(60));
  
  // Save deployment info
  const fs = require('fs');
  const deploymentInfo = {
    network: networkName,
    timestamp: new Date().toISOString(),
    deployer: deployer.address,
    contracts: {
      wrappedAxiom: wrappedToken.address,
      bridge: bridge.address
    }
  };
  
  const filename = `deployments/${networkName}-deployment.json`;
  fs.mkdirSync('deployments', { recursive: true });
  fs.writeFileSync(filename, JSON.stringify(deploymentInfo, null, 2));
  console.log(`\n💾 Deployment info saved to ${filename}`);

  // Verification instructions
  if (networkName !== 'hardhat' && networkName !== 'localhost') {
    console.log("\n🔍 To verify contracts, run:");
    console.log(`npx hardhat verify --network ${networkName} ${wrappedToken.address}`);
    console.log(`npx hardhat verify --network ${networkName} ${bridge.address} ${wrappedToken.address}`);
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
