import { apiRequest } from './api';
import { 
  User, 
  LoginRequest, 
  RegisterRequest, 
  AuthResponse,
  PasswordResetRequest,
  PasswordResetConfirm,
  ChangePasswordRequest
} from '../types/auth';

export const authService = {
  // Login user
  login: async (credentials: LoginRequest): Promise<AuthResponse> => {
    return apiRequest.post<AuthResponse>('/auth/login', credentials);
  },

  // Register new user
  register: async (userData: RegisterRequest): Promise<AuthResponse> => {
    return apiRequest.post<AuthResponse>('/auth/register', userData);
  },

  // Get current user
  getCurrentUser: async (): Promise<User> => {
    return apiRequest.get<User>('/auth/me');
  },

  // Refresh token
  refreshToken: async (): Promise<AuthResponse> => {
    return apiRequest.post<AuthResponse>('/auth/refresh');
  },

  // Logout user
  logout: async (): Promise<void> => {
    return apiRequest.post('/auth/logout');
  },

  // Request password reset
  requestPasswordReset: async (data: PasswordResetRequest): Promise<void> => {
    return apiRequest.post('/auth/password-reset', data);
  },

  // Confirm password reset
  confirmPasswordReset: async (data: PasswordResetConfirm): Promise<void> => {
    return apiRequest.post('/auth/password-reset/confirm', data);
  },

  // Change password
  changePassword: async (data: ChangePasswordRequest): Promise<void> => {
    return apiRequest.post('/auth/change-password', data);
  },

  // Verify email
  verifyEmail: async (token: string): Promise<void> => {
    return apiRequest.post('/auth/verify-email', { token });
  },

  // Resend verification email
  resendVerificationEmail: async (): Promise<void> => {
    return apiRequest.post('/auth/resend-verification');
  },

  // Update user profile
  updateProfile: async (profileData: Partial<User>): Promise<User> => {
    return apiRequest.patch<User>('/auth/profile', profileData);
  },

  // Upload avatar
  uploadAvatar: async (file: File): Promise<{ avatarUrl: string }> => {
    const formData = new FormData();
    formData.append('avatar', file);
    
    return apiRequest.post('/auth/avatar', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
  },

  // Delete account
  deleteAccount: async (password: string): Promise<void> => {
    return apiRequest.delete('/auth/account', {
      data: { password },
    });
  },

  // Enable 2FA
  enable2FA: async (): Promise<{ qrCode: string; secret: string }> => {
    return apiRequest.post('/auth/2fa/enable');
  },

  // Verify 2FA setup
  verify2FA: async (token: string): Promise<{ backupCodes: string[] }> => {
    return apiRequest.post('/auth/2fa/verify', { token });
  },

  // Disable 2FA
  disable2FA: async (token: string): Promise<void> => {
    return apiRequest.post('/auth/2fa/disable', { token });
  },

  // Generate new backup codes
  generateBackupCodes: async (): Promise<{ backupCodes: string[] }> => {
    return apiRequest.post('/auth/2fa/backup-codes');
  },
};