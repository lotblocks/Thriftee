import React, { useState } from 'react';
import {
  Box,
  Stack,
  TextField,
  Button,
  Typography,
  Alert,
  LinearProgress,
  Chip,
  Card,
  CardContent,
} from '@mui/material';
import {
  Lock,
  Visibility,
  VisibilityOff,
  CheckCircle,
  Cancel,
} from '@mui/icons-material';
import { useForm, Controller } from 'react-hook-form';
import { toast } from 'react-toastify';

import { authService } from '../../services/authService';

interface PasswordFormData {
  currentPassword: string;
  newPassword: string;
  confirmPassword: string;
}

interface PasswordStrength {
  score: number;
  label: string;
  color: 'error' | 'warning' | 'info' | 'success';
  requirements: {
    length: boolean;
    uppercase: boolean;
    lowercase: boolean;
    number: boolean;
    special: boolean;
  };
}

const PasswordChange: React.FC = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [showPasswords, setShowPasswords] = useState({
    current: false,
    new: false,
    confirm: false,
  });

  const {
    control,
    handleSubmit,
    watch,
    reset,
    formState: { errors },
  } = useForm<PasswordFormData>({
    defaultValues: {
      currentPassword: '',
      newPassword: '',
      confirmPassword: '',
    },
  });

  const newPassword = watch('newPassword');

  // Password strength calculation
  const calculatePasswordStrength = (password: string): PasswordStrength => {
    const requirements = {
      length: password.length >= 8,
      uppercase: /[A-Z]/.test(password),
      lowercase: /[a-z]/.test(password),
      number: /\d/.test(password),
      special: /[!@#$%^&*(),.?":{}|<>]/.test(password),
    };

    const score = Object.values(requirements).filter(Boolean).length;

    let label = 'Very Weak';
    let color: 'error' | 'warning' | 'info' | 'success' = 'error';

    if (score >= 5) {
      label = 'Very Strong';
      color = 'success';
    } else if (score >= 4) {
      label = 'Strong';
      color = 'success';
    } else if (score >= 3) {
      label = 'Medium';
      color = 'info';
    } else if (score >= 2) {
      label = 'Weak';
      color = 'warning';
    }

    return { score, label, color, requirements };
  };

  const passwordStrength = calculatePasswordStrength(newPassword || '');

  const togglePasswordVisibility = (field: keyof typeof showPasswords) => {
    setShowPasswords(prev => ({
      ...prev,
      [field]: !prev[field],
    }));
  };

  const onSubmit = async (data: PasswordFormData) => {
    setIsLoading(true);
    try {
      await authService.changePassword({
        currentPassword: data.currentPassword,
        newPassword: data.newPassword,
        confirmPassword: data.confirmPassword,
      });

      toast.success('Password changed successfully!');
      reset();
      setShowPasswords({ current: false, new: false, confirm: false });
    } catch (error: any) {
      console.error('Password change failed:', error);
      toast.error(error.response?.data?.message || 'Failed to change password');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <Card>
      <CardContent>
        <Stack spacing={4}>
          {/* Header */}
          <Box>
            <Stack direction="row" alignItems="center" spacing={2} mb={1}>
              <Lock color="primary" />
              <Typography variant="h6" fontWeight={600}>
                Change Password
              </Typography>
            </Stack>
            <Typography variant="body2" color="text.secondary">
              Update your password to keep your account secure
            </Typography>
          </Box>

          <form onSubmit={handleSubmit(onSubmit)}>
            <Stack spacing={3}>
              {/* Current Password */}
              <Controller
                name="currentPassword"
                control={control}
                rules={{ required: 'Current password is required' }}
                render={({ field }) => (
                  <TextField
                    {...field}
                    label="Current Password"
                    type={showPasswords.current ? 'text' : 'password'}
                    fullWidth
                    error={!!errors.currentPassword}
                    helperText={errors.currentPassword?.message}
                    InputProps={{
                      endAdornment: (
                        <Button
                          onClick={() => togglePasswordVisibility('current')}
                          sx={{ minWidth: 'auto', p: 1 }}
                        >
                          {showPasswords.current ? <VisibilityOff /> : <Visibility />}
                        </Button>
                      ),
                    }}
                  />
                )}
              />

              {/* New Password */}
              <Controller
                name="newPassword"
                control={control}
                rules={{
                  required: 'New password is required',
                  minLength: {
                    value: 8,
                    message: 'Password must be at least 8 characters',
                  },
                  validate: (value) => {
                    const strength = calculatePasswordStrength(value);
                    if (strength.score < 3) {
                      return 'Password is too weak. Please include uppercase, lowercase, numbers, and special characters.';
                    }
                    return true;
                  },
                }}
                render={({ field }) => (
                  <Box>
                    <TextField
                      {...field}
                      label="New Password"
                      type={showPasswords.new ? 'text' : 'password'}
                      fullWidth
                      error={!!errors.newPassword}
                      helperText={errors.newPassword?.message}
                      InputProps={{
                        endAdornment: (
                          <Button
                            onClick={() => togglePasswordVisibility('new')}
                            sx={{ minWidth: 'auto', p: 1 }}
                          >
                            {showPasswords.new ? <VisibilityOff /> : <Visibility />}
                          </Button>
                        ),
                      }}
                    />
                    
                    {/* Password strength indicator */}
                    {newPassword && (
                      <Box sx={{ mt: 2 }}>
                        <Stack direction="row" alignItems="center" spacing={2} mb={1}>
                          <Typography variant="body2" color="text.secondary">
                            Password Strength:
                          </Typography>
                          <Chip
                            label={passwordStrength.label}
                            color={passwordStrength.color}
                            size="small"
                          />
                        </Stack>
                        
                        <LinearProgress
                          variant="determinate"
                          value={(passwordStrength.score / 5) * 100}
                          color={passwordStrength.color}
                          sx={{ height: 6, borderRadius: 3, mb: 2 }}
                        />
                        
                        {/* Requirements checklist */}
                        <Stack spacing={1}>
                          {[
                            { key: 'length', label: 'At least 8 characters' },
                            { key: 'uppercase', label: 'One uppercase letter' },
                            { key: 'lowercase', label: 'One lowercase letter' },
                            { key: 'number', label: 'One number' },
                            { key: 'special', label: 'One special character' },
                          ].map((req) => (
                            <Stack key={req.key} direction="row" alignItems="center" spacing={1}>
                              {passwordStrength.requirements[req.key as keyof typeof passwordStrength.requirements] ? (
                                <CheckCircle sx={{ fontSize: 16, color: 'success.main' }} />
                              ) : (
                                <Cancel sx={{ fontSize: 16, color: 'error.main' }} />
                              )}
                              <Typography
                                variant="caption"
                                color={
                                  passwordStrength.requirements[req.key as keyof typeof passwordStrength.requirements]
                                    ? 'success.main'
                                    : 'text.secondary'
                                }
                              >
                                {req.label}
                              </Typography>
                            </Stack>
                          ))}
                        </Stack>
                      </Box>
                    )}
                  </Box>
                )}
              />

              {/* Confirm Password */}
              <Controller
                name="confirmPassword"
                control={control}
                rules={{
                  required: 'Please confirm your new password',
                  validate: (value) => {
                    if (value !== newPassword) {
                      return 'Passwords do not match';
                    }
                    return true;
                  },
                }}
                render={({ field }) => (
                  <TextField
                    {...field}
                    label="Confirm New Password"
                    type={showPasswords.confirm ? 'text' : 'password'}
                    fullWidth
                    error={!!errors.confirmPassword}
                    helperText={errors.confirmPassword?.message}
                    InputProps={{
                      endAdornment: (
                        <Button
                          onClick={() => togglePasswordVisibility('confirm')}
                          sx={{ minWidth: 'auto', p: 1 }}
                        >
                          {showPasswords.confirm ? <VisibilityOff /> : <Visibility />}
                        </Button>
                      ),
                    }}
                  />
                )}
              />

              {/* Security tips */}
              <Alert severity="info">
                <Typography variant="body2">
                  <strong>Security Tips:</strong>
                  <br />
                  • Use a unique password that you don't use elsewhere
                  <br />
                  • Consider using a password manager
                  <br />
                  • Enable two-factor authentication for extra security
                </Typography>
              </Alert>

              {/* Submit button */}
              <Button
                type="submit"
                variant="contained"
                disabled={isLoading || passwordStrength.score < 3}
                sx={{
                  background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                  '&:hover': {
                    background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                  },
                }}
              >
                {isLoading ? 'Changing Password...' : 'Change Password'}
              </Button>
            </Stack>
          </form>
        </Stack>
      </CardContent>
    </Card>
  );
};

export default PasswordChange;