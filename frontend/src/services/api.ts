import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios';
import toast from 'react-hot-toast';

// Create axios instance with base configuration
const api: AxiosInstance = axios.create({
  baseURL: import.meta.env.VITE_API_URL || 'http://localhost:8080/api/v1',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor to add auth token
api.interceptors.request.use(
  (config: any) => {
    const token = localStorage.getItem('auth_token');
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`;
    }
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor for error handling
api.interceptors.response.use(
  (response: AxiosResponse) => {
    return response;
  },
  (error) => {
    const { response } = error;

    if (response) {
      switch (response.status) {
        case 401:
          // Unauthorized - clear token and redirect to login
          localStorage.removeItem('auth_token');
          localStorage.removeItem('refresh_token');
          window.location.href = '/login';
          toast.error('Session expired. Please login again.');
          break;
        case 403:
          toast.error('Access denied. You do not have permission to perform this action.');
          break;
        case 404:
          toast.error('Resource not found.');
          break;
        case 422:
          // Validation errors
          if (response.data?.errors) {
            const errors = response.data.errors;
            Object.keys(errors).forEach(key => {
              toast.error(`${key}: ${errors[key].join(', ')}`);
            });
          } else {
            toast.error(response.data?.message || 'Validation error occurred.');
          }
          break;
        case 429:
          toast.error('Too many requests. Please try again later.');
          break;
        case 500:
          toast.error('Internal server error. Please try again later.');
          break;
        default:
          toast.error(response.data?.message || 'An unexpected error occurred.');
      }
    } else if (error.request) {
      // Network error
      toast.error('Network error. Please check your connection and try again.');
    } else {
      // Other error
      toast.error('An unexpected error occurred.');
    }

    return Promise.reject(error);
  }
);

// API helper functions
export const apiRequest = {
  get: <T = any>(url: string, config?: AxiosRequestConfig): Promise<T> =>
    api.get(url, config).then(response => response.data),

  post: <T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> =>
    api.post(url, data, config).then(response => response.data),

  put: <T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> =>
    api.put(url, data, config).then(response => response.data),

  patch: <T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> =>
    api.patch(url, data, config).then(response => response.data),

  delete: <T = any>(url: string, config?: AxiosRequestConfig): Promise<T> =>
    api.delete(url, config).then(response => response.data),
};

// File upload helper
export const uploadFile = async (
  url: string,
  file: File,
  onProgress?: (progress: number) => void
): Promise<any> => {
  const formData = new FormData();
  formData.append('file', file);

  return api.post(url, formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
    onUploadProgress: (progressEvent) => {
      if (onProgress && progressEvent.total) {
        const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total);
        onProgress(progress);
      }
    },
  }).then(response => response.data);
};

// WebSocket connection helper
export const createWebSocketConnection = (endpoint: string = '/ws'): WebSocket => {
  const wsUrl = import.meta.env.VITE_WS_URL || 'ws://localhost:8080';
  
  const ws = new WebSocket(`${wsUrl}${endpoint}`);
  
  ws.onopen = () => {
    console.log('WebSocket connected');
    
    // Authenticate the connection
    const token = localStorage.getItem('auth_token');
    if (token) {
      ws.send(JSON.stringify({
        message_type: 'auth',
        data: { token }
      }));
    }
  };
  
  ws.onclose = () => {
    console.log('WebSocket disconnected');
  };
  
  ws.onerror = (error) => {
    console.error('WebSocket error:', error);
    toast.error('Real-time connection error. Some features may not work properly.');
  };
  
  return ws;
};

export default api;