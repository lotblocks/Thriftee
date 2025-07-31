const { ethers } = require("ethers");

// Network configurations for different environments
const networks = {
  // Polygon Mumbai Testnet
  mumbai: {
    name: "Polygon Mumbai",
    chainId: 80001,
    rpcUrl: process.env.MUMBAI_RPC_URL || "https://rpc-mumbai.maticvigil.com",
    blockExplorer: "https://mumbai.polygonscan.com",
    nativeCurrency: {
      name: "MATIC",
      symbol: "MATIC",
      decimals: 18
    },
    vrf: {
      coordinator: "0x7a1BaC17Ccc5b313516C5E16fb24f7659aA5ebed",
      gasLane: "0x4b09e658ed251bcafeebbc69400383d49f344ace09b9576fe248bb02c003fe9f",
      subscriptionId: process.env.VRF_SUBSCRIPTION_ID_MUMBAI || "1",
      callbackGasLimit: "500000",
      linkToken: "0x326C977E6efc84E512bB9C30f76E30c160eD06FB"
    },
    gasPrice: ethers.utils.parseUnits("30", "gwei"),
    confirmations: 2
  },

  // Polygon Mainnet
  polygon: {
    name: "Polygon Mainnet",
    chainId: 137,
    rpcUrl: process.env.POLYGON_RPC_URL || "https://polygon-rpc.com",
    blockExplorer: "https://polygonscan.com",
    nativeCurrency: {
      name: "MATIC",
      symbol: "MATIC",
      decimals: 18
    },
    vrf: {
      coordinator: "0xAE975071Be8F8eE67addBC1A82488F1C24858067",
      gasLane: "0xcc294a196eeeb44da2888d17c0625cc88d70d9760a69d58d853ba6581a9ab0cd",
      subscriptionId: process.env.VRF_SUBSCRIPTION_ID_POLYGON || "1",
      callbackGasLimit: "500000",
      linkToken: "0xb0897686c545045aFc77CF20eC7A532E3120E0F1"
    },
    gasPrice: ethers.utils.parseUnits("50", "gwei"),
    confirmations: 5
  },

  // Ethereum Mainnet
  ethereum: {
    name: "Ethereum Mainnet",
    chainId: 1,
    rpcUrl: process.env.ETHEREUM_RPC_URL || "https://mainnet.infura.io/v3/" + process.env.INFURA_PROJECT_ID,
    blockExplorer: "https://etherscan.io",
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    },
    vrf: {
      coordinator: "0x271682DEB8C4E0901D1a1550aD2e64D568E69909",
      gasLane: "0x8af398995b04c28e9951adb9721ef74c74f93e6a478f39e7e0777be13527e7ef",
      subscriptionId: process.env.VRF_SUBSCRIPTION_ID_ETHEREUM || "1",
      callbackGasLimit: "500000",
      linkToken: "0x514910771AF9Ca656af840dff83E8264EcF986CA"
    },
    gasPrice: ethers.utils.parseUnits("20", "gwei"),
    confirmations: 6
  },

  // Hardhat Local Network
  hardhat: {
    name: "Hardhat Local",
    chainId: 31337,
    rpcUrl: "http://127.0.0.1:8545",
    blockExplorer: null,
    nativeCurrency: {
      name: "Ether",
      symbol: "ETH",
      decimals: 18
    },
    vrf: {
      coordinator: "0x0000000000000000000000000000000000000000", // Mock coordinator
      gasLane: "0x0000000000000000000000000000000000000000000000000000000000000000",
      subscriptionId: "1",
      callbackGasLimit: "500000",
      linkToken: "0x0000000000000000000000000000000000000000"
    },
    gasPrice: ethers.utils.parseUnits("8", "gwei"),
    confirmations: 1
  }
};

// Get network configuration by name or chain ID
function getNetworkConfig(networkName) {
  const config = networks[networkName];
  if (!config) {
    throw new Error(`Network configuration not found for: ${networkName}`);
  }
  return config;
}

// Get all available networks
function getAvailableNetworks() {
  return Object.keys(networks);
}

// Validate network configuration
function validateNetworkConfig(config) {
  const required = ['name', 'chainId', 'rpcUrl', 'vrf'];
  const vrfRequired = ['coordinator', 'gasLane', 'subscriptionId', 'callbackGasLimit'];
  
  for (const field of required) {
    if (!config[field]) {
      throw new Error(`Missing required network config field: ${field}`);
    }
  }
  
  for (const field of vrfRequired) {
    if (!config.vrf[field]) {
      throw new Error(`Missing required VRF config field: ${field}`);
    }
  }
  
  return true;
}

module.exports = {
  networks,
  getNetworkConfig,
  getAvailableNetworks,
  validateNetworkConfig
};