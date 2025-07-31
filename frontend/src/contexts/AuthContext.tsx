import React, { createContext, useContext, useEffect, useState } from 'react'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import toast from 'react-hot-toast'
import { api } from '@/services/api'

export interface User {
  id: string
  email: string
  role: 'user' | 'seller' | 'admin'
  isEmailVerified: boolean
  createdAt: string
  updatedAt: string
}

export interface AuthContextType {
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  login: (email: string, password: string) => Promise<void>
  register: (email: string, password: string, confirmPassword: string) => Promise<void>
  logout: () => Promise<void>
  refreshToken: () => Promise<void>
  updateUser: (userData: Partial<User>) => void
}

const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}

interface AuthProviderProps {
  children: React.ReactNode
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<User | null>(null)
  const [isInitialized, setIsInitialized] = useState(false)
  const queryClient = useQueryClient()

  // Get current user query
  const {
    data: currentUser,
    isLoading: isUserLoading,
    error: userError,
  } = useQuery({
    queryKey: ['auth', 'currentUser'],
    queryFn: async () => {
      const token = localStorage.getItem('auth_token')
      if (!token) {
        throw new Error('No token found')
      }
      
      const response = await api.get('/auth/me')
      return response.data
    },
    enabled: !!localStorage.getItem('auth_token') && isInitialized,
    retry: false,
    staleTime: 1000 * 60 * 5, // 5 minutes
  })

  // Login mutation
  const loginMutation = useMutation({
    mutationFn: async ({ email, password }: { email: string; password: string }) => {
      const response = await api.post('/auth/login', { email, password })
      return response.data
    },
    onSuccess: (data) => {
      localStorage.setItem('auth_token', data.token)
      localStorage.setItem('refresh_token', data.refreshToken)
      setUser(data.user)
      queryClient.setQueryData(['auth', 'currentUser'], data.user)
      
      // Set default authorization header
      api.defaults.headers.common['Authorization'] = `Bearer ${data.token}`
    },
    onError: (error: any) => {
      const message = error.response?.data?.message || 'Login failed'
      throw new Error(message)
    },
  })

  // Register mutation
  const registerMutation = useMutation({
    mutationFn: async ({ 
      email, 
      password, 
      confirmPassword 
    }: { 
      email: string
      password: string
      confirmPassword: string 
    }) => {
      const response = await api.post('/auth/register', {
        email,
        password,
        confirmPassword,
      })
      return response.data
    },
    onSuccess: (data) => {
      localStorage.setItem('auth_token', data.token)
      localStorage.setItem('refresh_token', data.refreshToken)
      setUser(data.user)
      queryClient.setQueryData(['auth', 'currentUser'], data.user)
      
      // Set default authorization header
      api.defaults.headers.common['Authorization'] = `Bearer ${data.token}`
    },
    onError: (error: any) => {
      const message = error.response?.data?.message || 'Registration failed'
      throw new Error(message)
    },
  })

  // Logout mutation
  const logoutMutation = useMutation({
    mutationFn: async () => {
      const token = localStorage.getItem('auth_token')
      if (token) {
        try {
          await api.post('/auth/logout')
        } catch (error) {
          // Ignore logout errors - we'll clear local state anyway
          console.warn('Logout request failed:', error)
        }
      }
    },
    onSettled: () => {
      // Clear local state regardless of API call success
      localStorage.removeItem('auth_token')
      localStorage.removeItem('refresh_token')
      delete api.defaults.headers.common['Authorization']
      setUser(null)
      queryClient.clear()
    },
  })

  // Refresh token mutation
  const refreshTokenMutation = useMutation({
    mutationFn: async () => {
      const refreshToken = localStorage.getItem('refresh_token')
      if (!refreshToken) {
        throw new Error('No refresh token found')
      }

      const response = await api.post('/auth/refresh', {
        refreshToken,
      })
      return response.data
    },
    onSuccess: (data) => {
      localStorage.setItem('auth_token', data.token)
      if (data.refreshToken) {
        localStorage.setItem('refresh_token', data.refreshToken)
      }
      
      // Set default authorization header
      api.defaults.headers.common['Authorization'] = `Bearer ${data.token}`
    },
    onError: () => {
      // Refresh failed, clear tokens and redirect to login
      localStorage.removeItem('auth_token')
      localStorage.removeItem('refresh_token')
      delete api.defaults.headers.common['Authorization']
      setUser(null)
      queryClient.clear()
    },
  })

  // Initialize auth state
  useEffect(() => {
    const token = localStorage.getItem('auth_token')
    if (token) {
      api.defaults.headers.common['Authorization'] = `Bearer ${token}`
    }
    setIsInitialized(true)
  }, [])

  // Update user state when query data changes
  useEffect(() => {
    if (currentUser) {
      setUser(currentUser)
    } else if (userError && isInitialized) {
      // Clear invalid tokens
      localStorage.removeItem('auth_token')
      localStorage.removeItem('refresh_token')
      delete api.defaults.headers.common['Authorization']
      setUser(null)
    }
  }, [currentUser, userError, isInitialized])

  // Set up axios interceptor for token refresh
  useEffect(() => {
    const interceptor = api.interceptors.response.use(
      (response) => response,
      async (error) => {
        const originalRequest = error.config

        if (
          error.response?.status === 401 &&
          !originalRequest._retry &&
          localStorage.getItem('refresh_token')
        ) {
          originalRequest._retry = true

          try {
            await refreshTokenMutation.mutateAsync()
            // Retry the original request with new token
            const token = localStorage.getItem('auth_token')
            if (token) {
              originalRequest.headers['Authorization'] = `Bearer ${token}`
              return api(originalRequest)
            }
          } catch (refreshError) {
            // Refresh failed, redirect to login
            return Promise.reject(refreshError)
          }
        }

        return Promise.reject(error)
      }
    )

    return () => {
      api.interceptors.response.eject(interceptor)
    }
  }, [refreshTokenMutation])

  const login = async (email: string, password: string) => {
    await loginMutation.mutateAsync({ email, password })
  }

  const register = async (email: string, password: string, confirmPassword: string) => {
    await registerMutation.mutateAsync({ email, password, confirmPassword })
  }

  const logout = async () => {
    await logoutMutation.mutateAsync()
  }

  const refreshToken = async () => {
    await refreshTokenMutation.mutateAsync()
  }

  const updateUser = (userData: Partial<User>) => {
    if (user) {
      const updatedUser = { ...user, ...userData }
      setUser(updatedUser)
      queryClient.setQueryData(['auth', 'currentUser'], updatedUser)
    }
  }

  const value: AuthContextType = {
    user,
    isLoading: !isInitialized || isUserLoading || loginMutation.isPending || registerMutation.isPending,
    isAuthenticated: !!user,
    login,
    register,
    logout,
    refreshToken,
    updateUser,
  }

  return (
    <AuthContext.Provider value={value}>
      {children}
    </AuthContext.Provider>
  )
}