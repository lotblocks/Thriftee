import React, { createContext, useContext, useEffect, useState, ReactNode } from 'react';
import { ethers } from 'ethers';
import { toast } from 'react-toastify';

interface Web3ContextType {
  provider: ethers.providers.Web3Provider | null;
  signer: ethers.Signer | null;
  account: string | null;
  chainId: number | null;
  isConnected: boolean;
  isConnecting: boolean;
  connectWallet: () => Promise<void>;
  disconnectWallet: () => void;
  switchNetwork: (chainId: number) => Promise<void>;
}

const Web3Context = createContext<Web3ContextType | undefined>(undefined);

interface Web3ProviderProps {
  children: ReactNode;
}

// Supported networks
const SUPPORTED_NETWORKS = {
  1: 'Ethereum Mainnet',
  137: 'Polygon Mainnet',
  80001: 'Polygon Mumbai Testnet',
  31337: 'Localhost',
};

export const Web3Provider: React.FC<Web3ProviderProps> = ({ children }) => {
  const [provider, setProvider] = useState<ethers.providers.Web3Provider | null>(null);
  const [signer, setSigner] = useState<ethers.Signer | null>(null);
  const [account, setAccount] = useState<string | null>(null);
  const [chainId, setChainId] = useState<number | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);

  // Check if MetaMask is installed
  const isMetaMaskInstalled = () => {
    return typeof window !== 'undefined' && typeof window.ethereum !== 'undefined';
  };

  // Initialize provider on mount
  useEffect(() => {
    if (isMetaMaskInstalled()) {
      const web3Provider = new ethers.providers.Web3Provider(window.ethereum);
      setProvider(web3Provider);

      // Check if already connected
      checkConnection();

      // Listen for account changes
      window.ethereum.on('accountsChanged', handleAccountsChanged);
      window.ethereum.on('chainChanged', handleChainChanged);
      window.ethereum.on('disconnect', handleDisconnect);

      return () => {
        window.ethereum.removeListener('accountsChanged', handleAccountsChanged);
        window.ethereum.removeListener('chainChanged', handleChainChanged);
        window.ethereum.removeListener('disconnect', handleDisconnect);
      };
    }
  }, []);

  const checkConnection = async () => {
    if (!isMetaMaskInstalled()) return;

    try {
      const accounts = await window.ethereum.request({ method: 'eth_accounts' });
      if (accounts.length > 0) {
        const web3Provider = new ethers.providers.Web3Provider(window.ethereum);
        const signer = web3Provider.getSigner();
        const network = await web3Provider.getNetwork();

        setProvider(web3Provider);
        setSigner(signer);
        setAccount(accounts[0]);
        setChainId(network.chainId);
        setIsConnected(true);
      }
    } catch (error) {
      console.error('Error checking connection:', error);
    }
  };

  const connectWallet = async () => {
    if (!isMetaMaskInstalled()) {
      toast.error('MetaMask is not installed. Please install MetaMask to continue.');
      window.open('https://metamask.io/download/', '_blank');
      return;
    }

    setIsConnecting(true);

    try {
      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts',
      });

      if (accounts.length > 0) {
        const web3Provider = new ethers.providers.Web3Provider(window.ethereum);
        const signer = web3Provider.getSigner();
        const network = await web3Provider.getNetwork();

        setProvider(web3Provider);
        setSigner(signer);
        setAccount(accounts[0]);
        setChainId(network.chainId);
        setIsConnected(true);

        toast.success('Wallet connected successfully!');

        // Check if on supported network
        if (!SUPPORTED_NETWORKS[network.chainId as keyof typeof SUPPORTED_NETWORKS]) {
          toast.warning('You are connected to an unsupported network. Please switch to a supported network.');
        }
      }
    } catch (error: any) {
      console.error('Error connecting wallet:', error);
      if (error.code === 4001) {
        toast.error('Connection rejected by user.');
      } else {
        toast.error('Failed to connect wallet. Please try again.');
      }
    } finally {
      setIsConnecting(false);
    }
  };

  const disconnectWallet = () => {
    setProvider(null);
    setSigner(null);
    setAccount(null);
    setChainId(null);
    setIsConnected(false);
    toast.info('Wallet disconnected.');
  };

  const switchNetwork = async (targetChainId: number) => {
    if (!isMetaMaskInstalled() || !window.ethereum) {
      toast.error('MetaMask is not available.');
      return;
    }

    try {
      await window.ethereum.request({
        method: 'wallet_switchEthereumChain',
        params: [{ chainId: `0x${targetChainId.toString(16)}` }],
      });
    } catch (error: any) {
      if (error.code === 4902) {
        // Network not added to MetaMask
        toast.error('Network not found. Please add the network to MetaMask manually.');
      } else {
        console.error('Error switching network:', error);
        toast.error('Failed to switch network.');
      }
    }
  };

  const handleAccountsChanged = (accounts: string[]) => {
    if (accounts.length === 0) {
      disconnectWallet();
    } else {
      setAccount(accounts[0]);
      toast.info('Account changed.');
    }
  };

  const handleChainChanged = (chainId: string) => {
    const newChainId = parseInt(chainId, 16);
    setChainId(newChainId);
    
    const networkName = SUPPORTED_NETWORKS[newChainId as keyof typeof SUPPORTED_NETWORKS];
    if (networkName) {
      toast.info(`Switched to ${networkName}.`);
    } else {
      toast.warning('Switched to unsupported network.');
    }
  };

  const handleDisconnect = () => {
    disconnectWallet();
  };

  const value: Web3ContextType = {
    provider,
    signer,
    account,
    chainId,
    isConnected,
    isConnecting,
    connectWallet,
    disconnectWallet,
    switchNetwork,
  };

  return <Web3Context.Provider value={value}>{children}</Web3Context.Provider>;
};

export const useWeb3 = (): Web3ContextType => {
  const context = useContext(Web3Context);
  if (context === undefined) {
    throw new Error('useWeb3 must be used within a Web3Provider');
  }
  return context;
};

// Extend Window interface for TypeScript
declare global {
  interface Window {
    ethereum?: any;
  }
}