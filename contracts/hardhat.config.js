require("@nomicfoundation/hardhat-toolbox");
require("@nomiclabs/hardhat-etherscan");
require("hardhat-gas-reporter");
require("solidity-coverage");
require("dotenv").config();

const { getNetworkConfig } = require("./config/networks");

// Get network configurations
const mumbaiConfig = getNetworkConfig("mumbai");
const polygonConfig = getNetworkConfig("polygon");
const ethereumConfig = getNetworkConfig("ethereum");

/** @type import('hardhat/config').HardhatUserConfig */
module.exports = {
  solidity: {
    version: "0.8.19",
    settings: {
      optimizer: {
        enabled: true,
        runs: 200,
      },
      viaIR: true,
    },
  },
  
  networks: {
    hardhat: {
      chainId: 31337,
      gas: 12000000,
      blockGasLimit: 12000000,
      allowUnlimitedContractSize: true,
    },
    
    mumbai: {
      url: mumbaiConfig.rpcUrl,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
      chainId: mumbaiConfig.chainId,
      gasPrice: mumbaiConfig.gasPrice.toNumber(),
      gas: 6000000,
      confirmations: mumbaiConfig.confirmations,
      timeoutBlocks: 200,
      skipDryRun: true
    },
    
    polygon: {
      url: polygonConfig.rpcUrl,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
      chainId: polygonConfig.chainId,
      gasPrice: polygonConfig.gasPrice.toNumber(),
      gas: 6000000,
      confirmations: polygonConfig.confirmations,
      timeoutBlocks: 200,
      skipDryRun: true
    },
    
    ethereum: {
      url: ethereumConfig.rpcUrl,
      accounts: process.env.PRIVATE_KEY ? [process.env.PRIVATE_KEY] : [],
      chainId: ethereumConfig.chainId,
      gasPrice: ethereumConfig.gasPrice.toNumber(),
      gas: 6000000,
      confirmations: ethereumConfig.confirmations,
      timeoutBlocks: 200,
      skipDryRun: true
    }
  },
  
  etherscan: {
    apiKey: {
      polygon: process.env.POLYGONSCAN_API_KEY,
      polygonMumbai: process.env.POLYGONSCAN_API_KEY,
      mainnet: process.env.ETHERSCAN_API_KEY,
    },
    customChains: [
      {
        network: "polygonMumbai",
        chainId: 80001,
        urls: {
          apiURL: "https://api-testnet.polygonscan.com/api",
          browserURL: "https://mumbai.polygonscan.com"
        }
      }
    ]
  },
  
  gasReporter: {
    enabled: process.env.REPORT_GAS !== undefined,
    currency: "USD",
    gasPrice: 30,
    coinmarketcap: process.env.COINMARKETCAP_API_KEY,
  },
  
  mocha: {
    timeout: 60000,
  },
  
  paths: {
    sources: "./contracts",
    tests: "./test",
    cache: "./cache",
    artifacts: "./artifacts"
  }
};