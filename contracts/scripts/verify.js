const { ethers } = require("hardhat");
const fs = require("fs");
const path = require("path");

async function main() {
  const network = hre.network.name;
  const deploymentFile = path.join(__dirname, "..", "deployments", `${network}.json`);

  if (!fs.existsSync(deploymentFile)) {
    throw new Error(`Deployment file not found for network: ${network}`);
  }

  const deploymentInfo = JSON.parse(fs.readFileSync(deploymentFile));
  const raffleContract = deploymentInfo.contracts.RaffleContract;

  console.log(`Verifying RaffleContract on ${network}...`);
  console.log(`Address: ${raffleContract.address}`);

  try {
    await hre.run("verify:verify", {
      address: raffleContract.address,
      constructorArguments: raffleContract.constructorArgs,
    });
    console.log("Contract verified successfully");
  } catch (error) {
    console.log("Verification failed:", error.message);
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });