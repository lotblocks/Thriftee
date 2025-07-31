// Real-time optimization utilities for WebSocket connections and updates

export interface RealtimeConfig {
  reconnectInterval: number;
  maxReconnectAttempts: number;
  heartbeatInterval: number;
  messageQueueSize: number;
  compressionEnabled: boolean;
  batchingEnabled: boolean;
  batchSize: number;
  batchTimeout: number;
  throttleUpdates: boolean;
  throttleInterval: number;
}

export interface RealtimeMessage {
  id: string;
  type: string;
  data: any;
  timestamp: number;
  priority: 'low' | 'normal' | 'high' | 'critical';
}

export interface ConnectionMetrics {
  connected: boolean;
  reconnectAttempts: number;
  lastConnected: number;
  messagesSent: number;
  messagesReceived: number;
  averageLatency: number;
  connectionQuality: 'poor' | 'fair' | 'good' | 'excellent';
}

class OptimizedWebSocket {
  private ws: WebSocket | null = null;
  private config: RealtimeConfig;
  private url: string;
  private protocols?: string | string[];
  private reconnectTimer?: NodeJS.Timeout;
  private heartbeatTimer?: NodeJS.Timeout;
  private batchTimer?: NodeJS.Timeout;
  private messageQueue: RealtimeMessage[] = [];
  private pendingMessages: Map<string, { resolve: Function; reject: Function; timestamp: number }> = new Map();
  private eventListeners: Map<string, Function[]> = new Map();
  private metrics: ConnectionMetrics = {
    connected: false,
    reconnectAttempts: 0,
    lastConnected: 0,
    messagesSent: 0,
    messagesReceived: 0,
    averageLatency: 0,
    connectionQuality: 'poor',
  };
  private latencyHistory: number[] = [];
  private updateThrottler: Map<string, { lastUpdate: number; pendingData: any }> = new Map();

  constructor(url: string, protocols?: string | string[], config: Partial<RealtimeConfig> = {}) {
    this.url = url;
    this.protocols = protocols;
    this.config = {
      reconnectInterval: 1000,
      maxReconnectAttempts: 10,
      heartbeatInterval: 30000,
      messageQueueSize: 1000,
      compressionEnabled: true,
      batchingEnabled: true,
      batchSize: 10,
      batchTimeout: 100,
      throttleUpdates: true,
      throttleInterval: 16, // ~60fps
      ...config,
    };

    this.connect();
  }

  private connect(): void {
    try {
      this.ws = new WebSocket(this.url, this.protocols);
      this.setupEventListeners();
    } catch (error) {
      console.error('Failed to create WebSocket connection:', error);
      this.scheduleReconnect();
    }
  }

  private setupEventListeners(): void {
    if (!this.ws) return;

    this.ws.onopen = (event) => {
      console.log('WebSocket connected');
      this.metrics.connected = true;
      this.metrics.reconnectAttempts = 0;
      this.metrics.lastConnected = Date.now();
      this.updateConnectionQuality();
      
      this.startHeartbeat();
      this.flushMessageQueue();
      this.emit('open', event);
    };

    this.ws.onclose = (event) => {
      console.log('WebSocket disconnected:', event.code, event.reason);
      this.metrics.connected = false;
      this.stopHeartbeat();
      this.emit('close', event);
      
      if (!event.wasClean && this.metrics.reconnectAttempts < this.config.maxReconnectAttempts) {
        this.scheduleReconnect();
      }
    };

    this.ws.onerror = (event) => {
      console.error('WebSocket error:', event);
      this.emit('error', event);
    };

    this.ws.onmessage = (event) => {
      this.handleMessage(event);
    };
  }

  private handleMessage(event: MessageEvent): void {
    try {
      let data = event.data;
      
      // Handle compression if enabled
      if (this.config.compressionEnabled && typeof data === 'string' && data.startsWith('compressed:')) {
        data = this.decompress(data.substring(11));
      }

      const message = JSON.parse(data);
      this.metrics.messagesReceived++;

      // Handle response messages (for request-response pattern)
      if (message.id && this.pendingMessages.has(message.id)) {
        const pending = this.pendingMessages.get(message.id)!;
        const latency = Date.now() - pending.timestamp;
        this.updateLatency(latency);
        
        if (message.error) {
          pending.reject(new Error(message.error));
        } else {
          pending.resolve(message.data);
        }
        
        this.pendingMessages.delete(message.id);
        return;
      }

      // Handle regular messages with throttling
      if (this.config.throttleUpdates && message.type) {
        this.throttleUpdate(message.type, message, () => {
          this.emit('message', message);
          this.emit(message.type, message.data);
        });
      } else {
        this.emit('message', message);
        if (message.type) {
          this.emit(message.type, message.data);
        }
      }
    } catch (error) {
      console.error('Failed to parse WebSocket message:', error);
    }
  }

  private throttleUpdate(type: string, message: any, callback: Function): void {
    const now = Date.now();
    const throttleData = this.updateThrottler.get(type);
    
    if (!throttleData || now - throttleData.lastUpdate >= this.config.throttleInterval) {
      // Execute immediately
      callback();
      this.updateThrottler.set(type, { lastUpdate: now, pendingData: null });
    } else {
      // Queue for later execution
      this.updateThrottler.set(type, { 
        lastUpdate: throttleData.lastUpdate, 
        pendingData: message 
      });
      
      // Schedule execution
      setTimeout(() => {
        const currentThrottleData = this.updateThrottler.get(type);
        if (currentThrottleData?.pendingData) {
          callback();
          this.updateThrottler.set(type, { 
            lastUpdate: Date.now(), 
            pendingData: null 
          });
        }
      }, this.config.throttleInterval - (now - throttleData.lastUpdate));
    }
  }

  private scheduleReconnect(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }

    const delay = Math.min(
      this.config.reconnectInterval * Math.pow(2, this.metrics.reconnectAttempts),
      30000 // Max 30 seconds
    );

    this.reconnectTimer = setTimeout(() => {
      this.metrics.reconnectAttempts++;
      console.log(`Attempting to reconnect (${this.metrics.reconnectAttempts}/${this.config.maxReconnectAttempts})`);
      this.connect();
    }, delay);
  }

  private startHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
    }

    this.heartbeatTimer = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.send({ type: 'ping', timestamp: Date.now() });
      }
    }, this.config.heartbeatInterval);
  }

  private stopHeartbeat(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer);
      this.heartbeatTimer = undefined;
    }
  }

  private flushMessageQueue(): void {
    while (this.messageQueue.length > 0 && this.ws?.readyState === WebSocket.OPEN) {
      const message = this.messageQueue.shift()!;
      this.sendMessage(message);
    }
  }

  private updateLatency(latency: number): void {
    this.latencyHistory.push(latency);
    if (this.latencyHistory.length > 100) {
      this.latencyHistory.shift();
    }
    
    this.metrics.averageLatency = this.latencyHistory.reduce((a, b) => a + b, 0) / this.latencyHistory.length;
    this.updateConnectionQuality();
  }

  private updateConnectionQuality(): void {
    const latency = this.metrics.averageLatency;
    
    if (latency < 50) {
      this.metrics.connectionQuality = 'excellent';
    } else if (latency < 150) {
      this.metrics.connectionQuality = 'good';
    } else if (latency < 300) {
      this.metrics.connectionQuality = 'fair';
    } else {
      this.metrics.connectionQuality = 'poor';
    }
  }

  private compress(data: string): string {
    // Simple compression implementation
    // In production, use a proper compression library
    return btoa(data);
  }

  private decompress(data: string): string {
    // Simple decompression implementation
    return atob(data);
  }

  private generateMessageId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  // Public API

  public send(data: any, priority: RealtimeMessage['priority'] = 'normal'): void {
    const message: RealtimeMessage = {
      id: this.generateMessageId(),
      type: data.type || 'message',
      data,
      timestamp: Date.now(),
      priority,
    };

    if (this.ws?.readyState === WebSocket.OPEN) {
      this.sendMessage(message);
    } else {
      this.queueMessage(message);
    }
  }

  public sendWithResponse<T = any>(data: any, timeout = 5000): Promise<T> {
    return new Promise((resolve, reject) => {
      const messageId = this.generateMessageId();
      const message = { ...data, id: messageId };

      this.pendingMessages.set(messageId, { resolve, reject, timestamp: Date.now() });

      // Set timeout
      setTimeout(() => {
        if (this.pendingMessages.has(messageId)) {
          this.pendingMessages.delete(messageId);
          reject(new Error('Request timeout'));
        }
      }, timeout);

      this.send(message, 'high');
    });
  }

  private sendMessage(message: RealtimeMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      this.queueMessage(message);
      return;
    }

    try {
      let payload = JSON.stringify(message.data);
      
      // Apply compression if enabled and beneficial
      if (this.config.compressionEnabled && payload.length > 1000) {
        const compressed = this.compress(payload);
        if (compressed.length < payload.length * 0.8) {
          payload = 'compressed:' + compressed;
        }
      }

      this.ws.send(payload);
      this.metrics.messagesSent++;
    } catch (error) {
      console.error('Failed to send WebSocket message:', error);
      this.queueMessage(message);
    }
  }

  private queueMessage(message: RealtimeMessage): void {
    // Sort by priority
    const priorityOrder = { critical: 0, high: 1, normal: 2, low: 3 };
    
    this.messageQueue.push(message);
    this.messageQueue.sort((a, b) => priorityOrder[a.priority] - priorityOrder[b.priority]);

    // Limit queue size
    if (this.messageQueue.length > this.config.messageQueueSize) {
      this.messageQueue = this.messageQueue.slice(0, this.config.messageQueueSize);
    }
  }

  public on(event: string, callback: Function): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, []);
    }
    this.eventListeners.get(event)!.push(callback);
  }

  public off(event: string, callback?: Function): void {
    if (!this.eventListeners.has(event)) return;
    
    if (callback) {
      const listeners = this.eventListeners.get(event)!;
      const index = listeners.indexOf(callback);
      if (index > -1) {
        listeners.splice(index, 1);
      }
    } else {
      this.eventListeners.delete(event);
    }
  }

  private emit(event: string, data?: any): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(data);
        } catch (error) {
          console.error(`Error in WebSocket event listener for ${event}:`, error);
        }
      });
    }
  }

  public getMetrics(): ConnectionMetrics {
    return { ...this.metrics };
  }

  public getConnectionState(): number {
    return this.ws?.readyState ?? WebSocket.CLOSED;
  }

  public close(code?: number, reason?: string): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
    }
    
    this.stopHeartbeat();
    
    if (this.ws) {
      this.ws.close(code, reason);
    }
  }

  public reconnect(): void {
    this.close();
    this.metrics.reconnectAttempts = 0;
    setTimeout(() => this.connect(), 100);
  }
}

// React hook for optimized WebSocket connections
export const useOptimizedWebSocket = (
  url: string,
  protocols?: string | string[],
  config?: Partial<RealtimeConfig>
) => {
  const [ws, setWs] = React.useState<OptimizedWebSocket | null>(null);
  const [connectionState, setConnectionState] = React.useState<number>(WebSocket.CLOSED);
  const [metrics, setMetrics] = React.useState<ConnectionMetrics>({
    connected: false,
    reconnectAttempts: 0,
    lastConnected: 0,
    messagesSent: 0,
    messagesReceived: 0,
    averageLatency: 0,
    connectionQuality: 'poor',
  });

  React.useEffect(() => {
    const websocket = new OptimizedWebSocket(url, protocols, config);
    setWs(websocket);

    const updateState = () => {
      setConnectionState(websocket.getConnectionState());
      setMetrics(websocket.getMetrics());
    };

    websocket.on('open', updateState);
    websocket.on('close', updateState);
    websocket.on('error', updateState);

    // Update metrics periodically
    const metricsInterval = setInterval(updateState, 1000);

    return () => {
      clearInterval(metricsInterval);
      websocket.close();
    };
  }, [url, protocols, config]);

  const send = React.useCallback((data: any, priority?: RealtimeMessage['priority']) => {
    ws?.send(data, priority);
  }, [ws]);

  const sendWithResponse = React.useCallback(<T = any>(data: any, timeout?: number): Promise<T> => {
    return ws?.sendWithResponse<T>(data, timeout) ?? Promise.reject(new Error('WebSocket not connected'));
  }, [ws]);

  const subscribe = React.useCallback((event: string, callback: Function) => {
    ws?.on(event, callback);
    return () => ws?.off(event, callback);
  }, [ws]);

  return {
    send,
    sendWithResponse,
    subscribe,
    connectionState,
    metrics,
    reconnect: () => ws?.reconnect(),
    close: (code?: number, reason?: string) => ws?.close(code, reason),
  };
};

// Bandwidth optimization utilities
export class BandwidthOptimizer {
  private compressionRatio = 0.7;
  private updateFrequency = new Map<string, number>();
  private lastUpdate = new Map<string, number>();

  public shouldSendUpdate(type: string, data: any, minInterval = 100): boolean {
    const now = Date.now();
    const lastUpdateTime = this.lastUpdate.get(type) || 0;
    
    if (now - lastUpdateTime < minInterval) {
      return false;
    }
    
    this.lastUpdate.set(type, now);
    return true;
  }

  public optimizePayload(data: any): any {
    // Remove unnecessary fields
    const optimized = { ...data };
    
    // Remove null/undefined values
    Object.keys(optimized).forEach(key => {
      if (optimized[key] == null) {
        delete optimized[key];
      }
    });
    
    // Truncate long strings
    Object.keys(optimized).forEach(key => {
      if (typeof optimized[key] === 'string' && optimized[key].length > 1000) {
        optimized[key] = optimized[key].substring(0, 1000) + '...';
      }
    });
    
    return optimized;
  }

  public estimatePayloadSize(data: any): number {
    return JSON.stringify(data).length;
  }

  public shouldCompress(data: any): boolean {
    const size = this.estimatePayloadSize(data);
    return size > 1000; // Compress payloads larger than 1KB
  }
}

// Connection pool for multiple WebSocket connections
export class WebSocketPool {
  private connections = new Map<string, OptimizedWebSocket>();
  private defaultConfig: RealtimeConfig;

  constructor(defaultConfig: Partial<RealtimeConfig> = {}) {
    this.defaultConfig = {
      reconnectInterval: 1000,
      maxReconnectAttempts: 10,
      heartbeatInterval: 30000,
      messageQueueSize: 1000,
      compressionEnabled: true,
      batchingEnabled: true,
      batchSize: 10,
      batchTimeout: 100,
      throttleUpdates: true,
      throttleInterval: 16,
      ...defaultConfig,
    };
  }

  public getConnection(
    name: string,
    url: string,
    protocols?: string | string[],
    config?: Partial<RealtimeConfig>
  ): OptimizedWebSocket {
    if (!this.connections.has(name)) {
      const connection = new OptimizedWebSocket(url, protocols, { ...this.defaultConfig, ...config });
      this.connections.set(name, connection);
    }
    
    return this.connections.get(name)!;
  }

  public closeConnection(name: string): void {
    const connection = this.connections.get(name);
    if (connection) {
      connection.close();
      this.connections.delete(name);
    }
  }

  public closeAllConnections(): void {
    this.connections.forEach((connection, name) => {
      connection.close();
    });
    this.connections.clear();
  }

  public getConnectionMetrics(): Map<string, ConnectionMetrics> {
    const metrics = new Map<string, ConnectionMetrics>();
    this.connections.forEach((connection, name) => {
      metrics.set(name, connection.getMetrics());
    });
    return metrics;
  }
}

export default OptimizedWebSocket;