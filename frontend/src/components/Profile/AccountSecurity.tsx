import React, { useState } from 'react';
import {
  Box,
  Stack,
  Typography,
  Button,
  Card,
  CardContent,
  Chip,
  Alert,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  Divider,
  QRCodeCanvas,
} from '@mui/material';
import {
  Security,
  PhoneAndroid,
  CheckCircle,
  Warning,
  Add,
  Delete,
  Smartphone,
  Computer,
  Tablet,
  LocationOn,
} from '@mui/icons-material';
import { toast } from 'react-toastify';

import { authService } from '../../services/authService';

interface TwoFactorSetup {
  qrCode: string;
  secret: string;
  backupCodes: string[];
}

interface LoginSession {
  id: string;
  device: string;
  location: string;
  ipAddress: string;
  lastActive: string;
  current: boolean;
}

const AccountSecurity: React.FC = () => {
  const [twoFactorEnabled, setTwoFactorEnabled] = useState(false);
  const [showTwoFactorDialog, setShowTwoFactorDialog] = useState(false);
  const [twoFactorSetup, setTwoFactorSetup] = useState<TwoFactorSetup | null>(null);
  const [verificationCode, setVerificationCode] = useState('');
  const [isLoading, setIsLoading] = useState(false);

  // Mock login sessions - in real app, this would come from API
  const loginSessions: LoginSession[] = [
    {
      id: '1',
      device: 'Chrome on Windows',
      location: 'New York, NY',
      ipAddress: '192.168.1.1',
      lastActive: '2024-01-18T15:30:00Z',
      current: true,
    },
    {
      id: '2',
      device: 'Safari on iPhone',
      location: 'New York, NY',
      ipAddress: '192.168.1.2',
      lastActive: '2024-01-17T09:20:00Z',
      current: false,
    },
    {
      id: '3',
      device: 'Chrome on Android',
      location: 'Brooklyn, NY',
      ipAddress: '192.168.1.3',
      lastActive: '2024-01-15T18:45:00Z',
      current: false,
    },
  ];

  const handleEnable2FA = async () => {
    setIsLoading(true);
    try {
      const setup = await authService.enable2FA();
      setTwoFactorSetup(setup);
      setShowTwoFactorDialog(true);
    } catch (error: any) {
      toast.error(error.response?.data?.message || 'Failed to enable 2FA');
    } finally {
      setIsLoading(false);
    }
  };

  const handleVerify2FA = async () => {
    if (!verificationCode || !twoFactorSetup) return;

    setIsLoading(true);
    try {
      const result = await authService.verify2FA(verificationCode);
      setTwoFactorEnabled(true);
      setShowTwoFactorDialog(false);
      setVerificationCode('');
      toast.success('Two-factor authentication enabled successfully!');
      
      // Show backup codes
      toast.info('Please save your backup codes in a secure location.');
    } catch (error: any) {
      toast.error(error.response?.data?.message || 'Invalid verification code');
    } finally {
      setIsLoading(false);
    }
  };

  const handleDisable2FA = async () => {
    const code = prompt('Enter your 2FA code to disable two-factor authentication:');
    if (!code) return;

    setIsLoading(true);
    try {
      await authService.disable2FA(code);
      setTwoFactorEnabled(false);
      toast.success('Two-factor authentication disabled');
    } catch (error: any) {
      toast.error(error.response?.data?.message || 'Failed to disable 2FA');
    } finally {
      setIsLoading(false);
    }
  };

  const handleTerminateSession = async (sessionId: string) => {
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 500));
      toast.success('Session terminated successfully');
    } catch (error) {
      toast.error('Failed to terminate session');
    }
  };

  const getDeviceIcon = (device: string) => {
    if (device.includes('iPhone') || device.includes('Android')) {
      return <Smartphone />;
    }
    if (device.includes('iPad') || device.includes('Tablet')) {
      return <Tablet />;
    }
    return <Computer />;
  };

  const formatLastActive = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffInHours = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60));
    
    if (diffInHours < 1) return 'Active now';
    if (diffInHours < 24) return `${diffInHours} hours ago`;
    
    const diffInDays = Math.floor(diffInHours / 24);
    return `${diffInDays} days ago`;
  };

  return (
    <>
      <Card>
        <CardContent>
          <Stack spacing={4}>
            {/* Header */}
            <Box>
              <Stack direction="row" alignItems="center" spacing={2} mb={1}>
                <Security color="primary" />
                <Typography variant="h6" fontWeight={600}>
                  Account Security
                </Typography>
              </Stack>
              <Typography variant="body2" color="text.secondary">
                Manage your account security settings and active sessions
              </Typography>
            </Box>

            {/* Two-Factor Authentication */}
            <Box>
              <Typography variant="subtitle1" fontWeight={600} gutterBottom>
                Two-Factor Authentication
              </Typography>
              
              <Card variant="outlined">
                <CardContent>
                  <Stack direction="row" alignItems="center" justifyContent="space-between">
                    <Box>
                      <Stack direction="row" alignItems="center" spacing={2} mb={1}>
                        <PhoneAndroid />
                        <Typography variant="body1" fontWeight={500}>
                          Authenticator App
                        </Typography>
                        <Chip
                          label={twoFactorEnabled ? 'Enabled' : 'Disabled'}
                          color={twoFactorEnabled ? 'success' : 'warning'}
                          size="small"
                        />
                      </Stack>
                      <Typography variant="body2" color="text.secondary">
                        {twoFactorEnabled
                          ? 'Your account is protected with two-factor authentication'
                          : 'Add an extra layer of security to your account'
                        }
                      </Typography>
                    </Box>
                    
                    <Button
                      variant={twoFactorEnabled ? 'outlined' : 'contained'}
                      color={twoFactorEnabled ? 'error' : 'primary'}
                      onClick={twoFactorEnabled ? handleDisable2FA : handleEnable2FA}
                      disabled={isLoading}
                      sx={!twoFactorEnabled ? {
                        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                        '&:hover': {
                          background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                        },
                      } : {}}
                    >
                      {twoFactorEnabled ? 'Disable' : 'Enable'}
                    </Button>
                  </Stack>
                </CardContent>
              </Card>

              {!twoFactorEnabled && (
                <Alert severity="warning" sx={{ mt: 2 }}>
                  <Typography variant="body2">
                    <strong>Recommended:</strong> Enable two-factor authentication to significantly improve your account security.
                    This adds an extra verification step when logging in from new devices.
                  </Typography>
                </Alert>
              )}
            </Box>

            {/* Active Sessions */}
            <Box>
              <Typography variant="subtitle1" fontWeight={600} gutterBottom>
                Active Sessions
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                These are the devices currently logged into your account. If you see any unfamiliar activity, 
                terminate the session and change your password immediately.
              </Typography>
              
              <Card variant="outlined">
                <List>
                  {loginSessions.map((session, index) => (
                    <React.Fragment key={session.id}>
                      <ListItem>
                        <ListItemIcon>
                          {getDeviceIcon(session.device)}
                        </ListItemIcon>
                        <ListItemText
                          primary={
                            <Stack direction="row" alignItems="center" spacing={1}>
                              <Typography variant="body1" fontWeight={500}>
                                {session.device}
                              </Typography>
                              {session.current && (
                                <Chip
                                  label="Current Session"
                                  color="success"
                                  size="small"
                                  sx={{ fontSize: '0.7rem', height: 20 }}
                                />
                              )}
                            </Stack>
                          }
                          secondary={
                            <Stack spacing={0.5} sx={{ mt: 0.5 }}>
                              <Stack direction="row" alignItems="center" spacing={1}>
                                <LocationOn sx={{ fontSize: 14 }} />
                                <Typography variant="caption">
                                  {session.location} • {session.ipAddress}
                                </Typography>
                              </Stack>
                              <Typography variant="caption" color="text.secondary">
                                {formatLastActive(session.lastActive)}
                              </Typography>
                            </Stack>
                          }
                        />
                        {!session.current && (
                          <Button
                            variant="outlined"
                            color="error"
                            size="small"
                            onClick={() => handleTerminateSession(session.id)}
                          >
                            Terminate
                          </Button>
                        )}
                      </ListItem>
                      {index < loginSessions.length - 1 && <Divider />}
                    </React.Fragment>
                  ))}
                </List>
              </Card>
            </Box>

            {/* Security Recommendations */}
            <Box>
              <Typography variant="subtitle1" fontWeight={600} gutterBottom>
                Security Recommendations
              </Typography>
              
              <Stack spacing={2}>
                <Alert severity="info">
                  <Typography variant="body2">
                    <strong>Keep your account secure:</strong>
                    <br />
                    • Use a strong, unique password
                    <br />
                    • Enable two-factor authentication
                    <br />
                    • Regularly review your active sessions
                    <br />
                    • Never share your login credentials
                    <br />
                    • Log out from public or shared devices
                  </Typography>
                </Alert>
              </Stack>
            </Box>
          </Stack>
        </CardContent>
      </Card>

      {/* 2FA Setup Dialog */}
      <Dialog
        open={showTwoFactorDialog}
        onClose={() => setShowTwoFactorDialog(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          <Stack direction="row" alignItems="center" spacing={2}>
            <Security color="primary" />
            <Typography variant="h6">Enable Two-Factor Authentication</Typography>
          </Stack>
        </DialogTitle>
        
        <DialogContent>
          {twoFactorSetup && (
            <Stack spacing={3} sx={{ pt: 1 }}>
              <Typography variant="body2">
                1. Install an authenticator app like Google Authenticator, Authy, or 1Password on your phone.
              </Typography>
              
              <Typography variant="body2">
                2. Scan this QR code with your authenticator app:
              </Typography>
              
              <Box sx={{ display: 'flex', justifyContent: 'center' }}>
                <QRCodeCanvas value={twoFactorSetup.qrCode} size={200} />
              </Box>
              
              <Typography variant="body2">
                3. Enter the 6-digit code from your authenticator app:
              </Typography>
              
              <TextField
                label="Verification Code"
                value={verificationCode}
                onChange={(e) => setVerificationCode(e.target.value)}
                placeholder="000000"
                inputProps={{ maxLength: 6 }}
                fullWidth
              />
              
              <Alert severity="warning">
                <Typography variant="body2">
                  <strong>Important:</strong> Save your backup codes in a secure location. 
                  You can use them to access your account if you lose your authenticator device.
                </Typography>
              </Alert>
            </Stack>
          )}
        </DialogContent>
        
        <DialogActions>
          <Button onClick={() => setShowTwoFactorDialog(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleVerify2FA}
            variant="contained"
            disabled={isLoading || verificationCode.length !== 6}
            sx={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              '&:hover': {
                background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
              },
            }}
          >
            {isLoading ? 'Verifying...' : 'Enable 2FA'}
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
};

export default AccountSecurity;