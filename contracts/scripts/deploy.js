const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

// Network configurations
const NETWORK_CONFIGS = {
  mumbai: {
    vrfCoordinator: "0x7a1BaC17Ccc5b313516C5E16fb24f7659aA5ebed",
    keyHash: "0x4b09e658ed251bcafeebbc69400383d49f344ace09b9576fe248bb02c003fe9f",
    subscriptionId: process.env.VRF_SUBSCRIPTION_ID || "1",
  },
  polygon: {
    vrfCoordinator: "0xAE975071Be8F8eE67addBC1A82488F1C24858067",
    keyHash: "0xcc294a196eeeb44da2888d17c0625cc88d70d9760a69d58d853ba6581a9ab0cd",
    subscriptionId: process.env.VRF_SUBSCRIPTION_ID || "1",
  },
  hardhat: {
    vrfCoordinator: "", // Will be set to mock coordinator
    keyHash: "0x4b09e658ed251bcafeebbc69400383d49f344ace09b9576fe248bb02c003fe9f",
    subscriptionId: "1",
  },
};

async function main() {
  const [deployer] = await ethers.getSigners();
  const network = hre.network.name;
  
  console.log("Deploying contracts with the account:", deployer.address);
  console.log("Network:", network);
  console.log("Account balance:", (await ethers.provider.getBalance(deployer.address)).toString());

  let config = NETWORK_CONFIGS[network];
  if (!config) {
    throw new Error(`Network ${network} not supported`);
  }

  let vrfCoordinatorAddress = config.vrfCoordinator;

  // Deploy mock VRF coordinator for local testing
  if (network === "hardhat" || network === "localhost") {
    console.log("Deploying MockVRFCoordinator for local testing...");
    const MockVRFCoordinator = await ethers.getContractFactory("MockVRFCoordinator");
    const mockVRFCoordinator = await MockVRFCoordinator.deploy();
    await mockVRFCoordinator.waitForDeployment();
    vrfCoordinatorAddress = await mockVRFCoordinator.getAddress();
    console.log("MockVRFCoordinator deployed to:", vrfCoordinatorAddress);
  }

  // Deploy RaffleContract
  console.log("Deploying RaffleContract...");
  const RaffleContract = await ethers.getContractFactory("RaffleContract");
  const raffleContract = await RaffleContract.deploy(
    vrfCoordinatorAddress,
    config.subscriptionId,
    config.keyHash
  );

  await raffleContract.waitForDeployment();
  const raffleContractAddress = await raffleContract.getAddress();

  console.log("RaffleContract deployed to:", raffleContractAddress);

  // Save deployment information
  const deploymentInfo = {
    network: network,
    chainId: (await ethers.provider.getNetwork()).chainId.toString(),
    deployer: deployer.address,
    contracts: {
      RaffleContract: {
        address: raffleContractAddress,
        constructorArgs: [
          vrfCoordinatorAddress,
          config.subscriptionId,
          config.keyHash,
        ],
      },
    },
    vrfConfig: {
      coordinator: vrfCoordinatorAddress,
      subscriptionId: config.subscriptionId,
      keyHash: config.keyHash,
    },
    deployedAt: new Date().toISOString(),
    blockNumber: await ethers.provider.getBlockNumber(),
  };

  // Create deployments directory if it doesn't exist
  const deploymentsDir = path.join(__dirname, "..", "deployments");
  if (!fs.existsSync(deploymentsDir)) {
    fs.mkdirSync(deploymentsDir, { recursive: true });
  }

  // Save deployment info
  const deploymentFile = path.join(deploymentsDir, `${network}.json`);
  fs.writeFileSync(deploymentFile, JSON.stringify(deploymentInfo, null, 2));

  console.log("Deployment info saved to:", deploymentFile);

  // Generate ABI files
  const artifactsDir = path.join(__dirname, "..", "artifacts", "contracts");
  const abiDir = path.join(__dirname, "..", "abi");
  
  if (!fs.existsSync(abiDir)) {
    fs.mkdirSync(abiDir, { recursive: true });
  }

  // Copy RaffleContract ABI
  const raffleArtifact = JSON.parse(
    fs.readFileSync(path.join(artifactsDir, "RaffleContract.sol", "RaffleContract.json"))
  );
  fs.writeFileSync(
    path.join(abiDir, "RaffleContract.json"),
    JSON.stringify(raffleArtifact.abi, null, 2)
  );

  console.log("ABI files generated in:", abiDir);

  // Verify contract on Etherscan (for testnets and mainnet)
  if (network !== "hardhat" && network !== "localhost") {
    console.log("Waiting for block confirmations...");
    await raffleContract.deploymentTransaction().wait(5);

    console.log("Verifying contract on Etherscan...");
    try {
      await hre.run("verify:verify", {
        address: raffleContractAddress,
        constructorArguments: [
          vrfCoordinatorAddress,
          config.subscriptionId,
          config.keyHash,
        ],
      });
      console.log("Contract verified successfully");
    } catch (error) {
      console.log("Verification failed:", error.message);
    }
  }

  console.log("\\n=== Deployment Summary ===");
  console.log(`Network: ${network}`);
  console.log(`RaffleContract: ${raffleContractAddress}`);
  console.log(`VRF Coordinator: ${vrfCoordinatorAddress}`);
  console.log(`Subscription ID: ${config.subscriptionId}`);
  console.log(`Key Hash: ${config.keyHash}`);
  console.log("===========================\\n");

  return {
    raffleContract,
    raffleContractAddress,
    deploymentInfo,
  };
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
if (require.main === module) {
  main()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(error);
      process.exit(1);
    });
}

module.exports = main;