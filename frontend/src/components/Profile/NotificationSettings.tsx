import React, { useState } from 'react';
import {
  Box,
  Stack,
  Typography,
  Switch,
  FormControlLabel,
  Card,
  CardContent,
  Divider,
  Button,
  Alert,
  Chip,
} from '@mui/material';
import {
  Notifications,
  Email,
  Sms,
  PhoneAndroid,
  Casino,
  EmojiEvents,
  AccountBalanceWallet,
  LocalShipping,
  Security,
  Campaign,
} from '@mui/icons-material';
import { toast } from 'react-toastify';

interface NotificationPreferences {
  email: {
    raffleUpdates: boolean;
    winnerAnnouncements: boolean;
    creditUpdates: boolean;
    shippingUpdates: boolean;
    securityAlerts: boolean;
    promotions: boolean;
    newsletter: boolean;
  };
  push: {
    raffleUpdates: boolean;
    winnerAnnouncements: boolean;
    creditUpdates: boolean;
    shippingUpdates: boolean;
    securityAlerts: boolean;
    promotions: boolean;
  };
  sms: {
    winnerAnnouncements: boolean;
    securityAlerts: boolean;
    shippingUpdates: boolean;
  };
}

const NotificationSettings: React.FC = () => {
  const [isLoading, setIsLoading] = useState(false);
  const [preferences, setPreferences] = useState<NotificationPreferences>({
    email: {
      raffleUpdates: true,
      winnerAnnouncements: true,
      creditUpdates: true,
      shippingUpdates: true,
      securityAlerts: true,
      promotions: false,
      newsletter: false,
    },
    push: {
      raffleUpdates: true,
      winnerAnnouncements: true,
      creditUpdates: false,
      shippingUpdates: true,
      securityAlerts: true,
      promotions: false,
    },
    sms: {
      winnerAnnouncements: true,
      securityAlerts: true,
      shippingUpdates: false,
    },
  });

  const handlePreferenceChange = (
    category: keyof NotificationPreferences,
    setting: string,
    value: boolean
  ) => {
    setPreferences(prev => ({
      ...prev,
      [category]: {
        ...prev[category],
        [setting]: value,
      },
    }));
  };

  const handleSave = async () => {
    setIsLoading(true);
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      toast.success('Notification preferences saved successfully!');
    } catch (error) {
      toast.error('Failed to save notification preferences');
    } finally {
      setIsLoading(false);
    }
  };

  const notificationCategories = [
    {
      title: 'Email Notifications',
      icon: <Email />,
      description: 'Receive updates via email',
      category: 'email' as const,
      settings: [
        {
          key: 'raffleUpdates',
          label: 'Raffle Updates',
          description: 'Box purchases, raffle progress, and completion notifications',
          icon: <Casino />,
          recommended: true,
        },
        {
          key: 'winnerAnnouncements',
          label: 'Winner Announcements',
          description: 'Notifications when you win or when raffles you participated in end',
          icon: <EmojiEvents />,
          recommended: true,
        },
        {
          key: 'creditUpdates',
          label: 'Credit Updates',
          description: 'Credit balance changes, expiration reminders, and redemption confirmations',
          icon: <AccountBalanceWallet />,
          recommended: true,
        },
        {
          key: 'shippingUpdates',
          label: 'Shipping Updates',
          description: 'Order confirmations, shipping notifications, and delivery updates',
          icon: <LocalShipping />,
          recommended: true,
        },
        {
          key: 'securityAlerts',
          label: 'Security Alerts',
          description: 'Login attempts, password changes, and account security notifications',
          icon: <Security />,
          recommended: true,
          required: true,
        },
        {
          key: 'promotions',
          label: 'Promotions & Offers',
          description: 'Special deals, discounts, and promotional campaigns',
          icon: <Campaign />,
          recommended: false,
        },
        {
          key: 'newsletter',
          label: 'Newsletter',
          description: 'Weekly digest of platform updates and featured raffles',
          icon: <Email />,
          recommended: false,
        },
      ],
    },
    {
      title: 'Push Notifications',
      icon: <PhoneAndroid />,
      description: 'Receive instant notifications on your device',
      category: 'push' as const,
      settings: [
        {
          key: 'raffleUpdates',
          label: 'Raffle Updates',
          description: 'Real-time updates on raffles you\'re participating in',
          icon: <Casino />,
          recommended: true,
        },
        {
          key: 'winnerAnnouncements',
          label: 'Winner Announcements',
          description: 'Instant notifications when you win',
          icon: <EmojiEvents />,
          recommended: true,
        },
        {
          key: 'creditUpdates',
          label: 'Credit Updates',
          description: 'Important credit balance and expiration notifications',
          icon: <AccountBalanceWallet />,
          recommended: false,
        },
        {
          key: 'shippingUpdates',
          label: 'Shipping Updates',
          description: 'Delivery notifications and tracking updates',
          icon: <LocalShipping />,
          recommended: true,
        },
        {
          key: 'securityAlerts',
          label: 'Security Alerts',
          description: 'Critical security notifications',
          icon: <Security />,
          recommended: true,
          required: true,
        },
        {
          key: 'promotions',
          label: 'Promotions',
          description: 'Limited-time offers and flash sales',
          icon: <Campaign />,
          recommended: false,
        },
      ],
    },
    {
      title: 'SMS Notifications',
      icon: <Sms />,
      description: 'Receive text messages for critical updates',
      category: 'sms' as const,
      settings: [
        {
          key: 'winnerAnnouncements',
          label: 'Winner Announcements',
          description: 'SMS when you win a raffle',
          icon: <EmojiEvents />,
          recommended: true,
        },
        {
          key: 'securityAlerts',
          label: 'Security Alerts',
          description: 'Critical security notifications via SMS',
          icon: <Security />,
          recommended: true,
          required: true,
        },
        {
          key: 'shippingUpdates',
          label: 'Shipping Updates',
          description: 'Delivery confirmations and important shipping updates',
          icon: <LocalShipping />,
          recommended: false,
        },
      ],
    },
  ];

  return (
    <Box>
      <Stack spacing={4}>
        {/* Header */}
        <Box>
          <Typography variant="body1" color="text.secondary" gutterBottom>
            Choose how you want to be notified about important updates and activities on your account.
          </Typography>
        </Box>

        {/* Notification categories */}
        {notificationCategories.map((category) => (
          <Card key={category.category}>
            <CardContent>
              <Stack spacing={3}>
                {/* Category header */}
                <Box>
                  <Stack direction="row" alignItems="center" spacing={2} mb={1}>
                    {category.icon}
                    <Typography variant="h6" fontWeight={600}>
                      {category.title}
                    </Typography>
                  </Stack>
                  <Typography variant="body2" color="text.secondary">
                    {category.description}
                  </Typography>
                </Box>

                <Divider />

                {/* Settings */}
                <Stack spacing={2}>
                  {category.settings.map((setting) => (
                    <Box key={setting.key}>
                      <Stack direction="row" alignItems="flex-start" spacing={2}>
                        <Box sx={{ mt: 0.5 }}>
                          {setting.icon}
                        </Box>
                        
                        <Box sx={{ flex: 1 }}>
                          <Stack direction="row" alignItems="center" spacing={1} mb={0.5}>
                            <Typography variant="subtitle1" fontWeight={500}>
                              {setting.label}
                            </Typography>
                            {setting.recommended && (
                              <Chip
                                label="Recommended"
                                size="small"
                                color="primary"
                                variant="outlined"
                                sx={{ fontSize: '0.7rem', height: 20 }}
                              />
                            )}
                            {setting.required && (
                              <Chip
                                label="Required"
                                size="small"
                                color="error"
                                sx={{ fontSize: '0.7rem', height: 20 }}
                              />
                            )}
                          </Stack>
                          <Typography variant="body2" color="text.secondary">
                            {setting.description}
                          </Typography>
                        </Box>

                        <FormControlLabel
                          control={
                            <Switch
                              checked={preferences[category.category][setting.key as keyof typeof preferences[typeof category.category]]}
                              onChange={(e) =>
                                handlePreferenceChange(category.category, setting.key, e.target.checked)
                              }
                              disabled={setting.required}
                            />
                          }
                          label=""
                          sx={{ m: 0 }}
                        />
                      </Stack>
                    </Box>
                  ))}
                </Stack>
              </Stack>
            </CardContent>
          </Card>
        ))}

        {/* Important notice */}
        <Alert severity="info">
          <Typography variant="body2">
            <strong>Note:</strong> Security alerts are required for account safety and cannot be disabled.
            You can unsubscribe from promotional emails at any time using the link in the email footer.
          </Typography>
        </Alert>

        {/* Save button */}
        <Box sx={{ display: 'flex', justifyContent: 'flex-end' }}>
          <Button
            variant="contained"
            onClick={handleSave}
            disabled={isLoading}
            sx={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              '&:hover': {
                background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
              },
            }}
          >
            {isLoading ? 'Saving...' : 'Save Preferences'}
          </Button>
        </Box>
      </Stack>
    </Box>
  );
};

export default NotificationSettings;