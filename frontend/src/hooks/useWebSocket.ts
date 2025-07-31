import { useEffect, useRef, useState, useCallback } from 'react';
import { toast } from 'react-toastify';

interface WebSocketMessage {
  type: string;
  data: any;
  timestamp: string;
}

interface UseWebSocketOptions {
  onMessage?: (message: WebSocketMessage) => void;
  onConnect?: () => void;
  onDisconnect?: () => void;
  onError?: (error: Event) => void;
  reconnectAttempts?: number;
  reconnectInterval?: number;
}

interface UseWebSocketReturn {
  isConnected: boolean;
  isConnecting: boolean;
  sendMessage: (type: string, data: any) => void;
  disconnect: () => void;
  reconnect: () => void;
}

export const useWebSocket = (
  endpoint: string,
  options: UseWebSocketOptions = {}
): UseWebSocketReturn => {
  const {
    onMessage,
    onConnect,
    onDisconnect,
    onError,
    reconnectAttempts = 5,
    reconnectInterval = 3000,
  } = options;

  const [isConnected, setIsConnected] = useState(false);
  const [isConnecting, setIsConnecting] = useState(false);
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectCountRef = useRef(0);
  const shouldReconnectRef = useRef(true);

  // Get WebSocket URL
  const getWebSocketUrl = useCallback(() => {
    const wsUrl = process.env.REACT_APP_WS_URL || 'ws://localhost:8080';
    const token = localStorage.getItem('token');
    return `${wsUrl}${endpoint}${token ? `?token=${token}` : ''}`;
  }, [endpoint]);

  // Send message
  const sendMessage = useCallback((type: string, data: any) => {
    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      const message: WebSocketMessage = {
        type,
        data,
        timestamp: new Date().toISOString(),
      };
      wsRef.current.send(JSON.stringify(message));
    } else {
      console.warn('WebSocket is not connected. Cannot send message:', { type, data });
    }
  }, []);

  // Connect to WebSocket
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.CONNECTING || 
        wsRef.current?.readyState === WebSocket.OPEN) {
      return;
    }

    setIsConnecting(true);
    shouldReconnectRef.current = true;

    try {
      const ws = new WebSocket(getWebSocketUrl());
      wsRef.current = ws;

      ws.onopen = () => {
        console.log('WebSocket connected to:', endpoint);
        setIsConnected(true);
        setIsConnecting(false);
        reconnectCountRef.current = 0;
        onConnect?.();
      };

      ws.onmessage = (event) => {
        try {
          const message: WebSocketMessage = JSON.parse(event.data);
          onMessage?.(message);
        } catch (error) {
          console.error('Failed to parse WebSocket message:', error);
        }
      };

      ws.onclose = (event) => {
        console.log('WebSocket disconnected:', event.code, event.reason);
        setIsConnected(false);
        setIsConnecting(false);
        wsRef.current = null;
        onDisconnect?.();

        // Attempt to reconnect if it wasn't a manual disconnect
        if (shouldReconnectRef.current && reconnectCountRef.current < reconnectAttempts) {
          const delay = reconnectInterval * Math.pow(1.5, reconnectCountRef.current);
          console.log(`Attempting to reconnect in ${delay}ms... (${reconnectCountRef.current + 1}/${reconnectAttempts})`);
          
          reconnectTimeoutRef.current = setTimeout(() => {
            reconnectCountRef.current++;
            connect();
          }, delay);
        } else if (reconnectCountRef.current >= reconnectAttempts) {
          toast.error('Lost connection to server. Please refresh the page.');
        }
      };

      ws.onerror = (error) => {
        console.error('WebSocket error:', error);
        setIsConnecting(false);
        onError?.(error);
      };

    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      setIsConnecting(false);
    }
  }, [endpoint, getWebSocketUrl, onConnect, onMessage, onDisconnect, onError, reconnectAttempts, reconnectInterval]);

  // Disconnect from WebSocket
  const disconnect = useCallback(() => {
    shouldReconnectRef.current = false;
    
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (wsRef.current) {
      wsRef.current.close(1000, 'Manual disconnect');
      wsRef.current = null;
    }

    setIsConnected(false);
    setIsConnecting(false);
  }, []);

  // Reconnect manually
  const reconnect = useCallback(() => {
    disconnect();
    setTimeout(() => {
      reconnectCountRef.current = 0;
      connect();
    }, 100);
  }, [disconnect, connect]);

  // Connect on mount
  useEffect(() => {
    connect();

    // Cleanup on unmount
    return () => {
      shouldReconnectRef.current = false;
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
      }
      if (wsRef.current) {
        wsRef.current.close(1000, 'Component unmount');
      }
    };
  }, [connect]);

  // Handle page visibility changes
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible' && !isConnected && shouldReconnectRef.current) {
        // Page became visible and we're not connected, try to reconnect
        reconnect();
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
    };
  }, [isConnected, reconnect]);

  return {
    isConnected,
    isConnecting,
    sendMessage,
    disconnect,
    reconnect,
  };
};