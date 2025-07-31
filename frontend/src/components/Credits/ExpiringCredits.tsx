import React, { useState } from 'react';
import {
  Box,
  Stack,
  Typography,
  Card,
  CardContent,
  Button,
  Alert,
  LinearProgress,
  Chip,
  Grid,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  List,
  ListItem,
  ListItemText,
  ListItemIcon,
  Divider,
} from '@mui/material';
import {
  Schedule,
  Warning,
  LocalOffer,
  CheckCircle,
  Error,
  Info,
} from '@mui/icons-material';
import { toast } from 'react-toastify';

interface ExpiringCredit {
  id: string;
  amount: number;
  type: 'general' | 'item_specific';
  itemName?: string;
  expiresAt: string;
  daysUntilExpiry: number;
  canRedeem: boolean;
}

const ExpiringCredits: React.FC = () => {
  const [showRedemptionDialog, setShowRedemptionDialog] = useState(false);
  const [selectedCredits, setSelectedCredits] = useState<ExpiringCredit[]>([]);

  // Mock expiring credits - in real app, this would come from API
  const expiringCredits: ExpiringCredit[] = [
    {
      id: '1',
      amount: 15,
      type: 'general',
      expiresAt: '2024-01-25T00:00:00Z',
      daysUntilExpiry: 7,
      canRedeem: true,
    },
    {
      id: '2',
      amount: 25,
      type: 'item_specific',
      itemName: 'iPhone 15 Pro Max',
      expiresAt: '2024-01-30T00:00:00Z',
      daysUntilExpiry: 12,
      canRedeem: true,
    },
    {
      id: '3',
      amount: 8,
      type: 'general',
      expiresAt: '2024-01-22T00:00:00Z',
      daysUntilExpiry: 4,
      canRedeem: true,
    },
    {
      id: '4',
      amount: 12,
      type: 'item_specific',
      itemName: 'MacBook Pro 16"',
      expiresAt: '2024-02-05T00:00:00Z',
      daysUntilExpiry: 18,
      canRedeem: false, // Maybe item no longer available
    },
  ];

  const totalExpiringAmount = expiringCredits.reduce((sum, credit) => sum + credit.amount, 0);
  const redeemableAmount = expiringCredits
    .filter(credit => credit.canRedeem)
    .reduce((sum, credit) => sum + credit.amount, 0);

  const getUrgencyLevel = (daysUntilExpiry: number) => {
    if (daysUntilExpiry <= 3) return 'critical';
    if (daysUntilExpiry <= 7) return 'warning';
    if (daysUntilExpiry <= 14) return 'info';
    return 'normal';
  };

  const getUrgencyColor = (urgency: string) => {
    switch (urgency) {
      case 'critical': return 'error';
      case 'warning': return 'warning';
      case 'info': return 'info';
      default: return 'default';
    }
  };

  const formatExpiryDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  };

  const handleRedeemAll = () => {
    const redeemableCredits = expiringCredits.filter(credit => credit.canRedeem);
    setSelectedCredits(redeemableCredits);
    setShowRedemptionDialog(true);
  };

  const handleRedeemCredit = (credit: ExpiringCredit) => {
    setSelectedCredits([credit]);
    setShowRedemptionDialog(true);
  };

  const confirmRedemption = async () => {
    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const totalAmount = selectedCredits.reduce((sum, credit) => sum + credit.amount, 0);
      toast.success(`Successfully redeemed $${totalAmount} in expiring credits!`);
      
      setShowRedemptionDialog(false);
      setSelectedCredits([]);
    } catch (error) {
      toast.error('Failed to redeem credits. Please try again.');
    }
  };

  return (
    <Box>
      <Stack spacing={4}>
        {/* Header */}
        <Box>
          <Stack direction="row" alignItems="center" spacing={2} mb={2}>
            <Schedule color="primary" sx={{ fontSize: 32 }} />
            <Box>
              <Typography variant="h5" fontWeight={600}>
                Expiring Credits
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Use your credits before they expire or redeem them for free items
              </Typography>
            </Box>
          </Stack>
        </Box>

        {/* Summary cards */}
        <Grid container spacing={3}>
          <Grid item xs={12} md={4}>
            <Card>
              <CardContent>
                <Stack alignItems="center" spacing={2}>
                  <Warning sx={{ fontSize: 48, color: 'warning.main' }} />
                  <Box textAlign="center">
                    <Typography variant="h4" fontWeight={700} color="warning.main">
                      ${totalExpiringAmount}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Total Expiring Credits
                    </Typography>
                  </Box>
                </Stack>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={4}>
            <Card>
              <CardContent>
                <Stack alignItems="center" spacing={2}>
                  <LocalOffer sx={{ fontSize: 48, color: 'success.main' }} />
                  <Box textAlign="center">
                    <Typography variant="h4" fontWeight={700} color="success.main">
                      ${redeemableAmount}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Available for Redemption
                    </Typography>
                  </Box>
                </Stack>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={4}>
            <Card>
              <CardContent>
                <Stack alignItems="center" spacing={2}>
                  <Schedule sx={{ fontSize: 48, color: 'info.main' }} />
                  <Box textAlign="center">
                    <Typography variant="h4" fontWeight={700} color="info.main">
                      {expiringCredits.length}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Credits Expiring Soon
                    </Typography>
                  </Box>
                </Stack>
              </CardContent>
            </Card>
          </Grid>
        </Grid>

        {/* Action buttons */}
        {redeemableAmount > 0 && (
          <Card>
            <CardContent>
              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Box>
                  <Typography variant="h6" fontWeight={600}>
                    Quick Actions
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Redeem all your expiring credits at once for free items
                  </Typography>
                </Box>
                <Button
                  variant="contained"
                  size="large"
                  onClick={handleRedeemAll}
                  sx={{
                    background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                    '&:hover': {
                      background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                    },
                  }}
                >
                  Redeem All (${redeemableAmount})
                </Button>
              </Stack>
            </CardContent>
          </Card>
        )}

        {/* Expiring credits list */}
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom fontWeight={600}>
              Credits Expiring Soon
            </Typography>
            
            <Stack spacing={2}>
              {expiringCredits.map((credit, index) => {
                const urgency = getUrgencyLevel(credit.daysUntilExpiry);
                const urgencyColor = getUrgencyColor(urgency);
                
                return (
                  <React.Fragment key={credit.id}>
                    <Card variant="outlined">
                      <CardContent>
                        <Stack direction="row" justifyContent="space-between" alignItems="center">
                          <Box sx={{ flex: 1 }}>
                            <Stack direction="row" alignItems="center" spacing={2} mb={1}>
                              <Typography variant="h6" fontWeight={600} color="primary">
                                ${credit.amount}
                              </Typography>
                              <Chip
                                label={credit.type === 'general' ? 'General' : 'Item-Specific'}
                                size="small"
                                color={credit.type === 'general' ? 'primary' : 'secondary'}
                                variant="outlined"
                              />
                              <Chip
                                label={`${credit.daysUntilExpiry} days left`}
                                size="small"
                                color={urgencyColor as any}
                                variant="filled"
                              />
                            </Stack>
                            
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                              {credit.type === 'item_specific' && credit.itemName
                                ? `Specific to: ${credit.itemName}`
                                : 'Can be used for any raffle or redemption'
                              }
                            </Typography>
                            
                            <Typography variant="body2" color="text.secondary">
                              Expires: {formatExpiryDate(credit.expiresAt)}
                            </Typography>

                            {/* Progress bar showing time until expiry */}
                            <Box sx={{ mt: 2 }}>
                              <LinearProgress
                                variant="determinate"
                                value={Math.max(0, Math.min(100, (30 - credit.daysUntilExpiry) / 30 * 100))}
                                color={urgencyColor as any}
                                sx={{ height: 6, borderRadius: 3 }}
                              />
                            </Box>
                          </Box>

                          <Stack spacing={1} alignItems="flex-end">
                            {credit.canRedeem ? (
                              <Button
                                variant="outlined"
                                size="small"
                                onClick={() => handleRedeemCredit(credit)}
                              >
                                Redeem Now
                              </Button>
                            ) : (
                              <Chip
                                label="Cannot Redeem"
                                size="small"
                                color="error"
                                variant="outlined"
                              />
                            )}
                          </Stack>
                        </Stack>
                      </CardContent>
                    </Card>
                    
                    {index < expiringCredits.length - 1 && <Divider />}
                  </React.Fragment>
                );
              })}
            </Stack>
          </CardContent>
        </Card>

        {expiringCredits.length === 0 && (
          <Card>
            <CardContent>
              <Box sx={{ textAlign: 'center', py: 6 }}>
                <CheckCircle sx={{ fontSize: 64, color: 'success.main', mb: 2 }} />
                <Typography variant="h6" gutterBottom>
                  No Credits Expiring Soon
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  All your credits are safe! Check back later or view your full credit balance.
                </Typography>
              </Box>
            </CardContent>
          </Card>
        )}

        {/* Information section */}
        <Alert severity="info" icon={<Info />}>
          <Typography variant="body2">
            <strong>Credit Expiration Policy:</strong>
            <br />
            • General credits expire 90 days after earning
            <br />
            • Item-specific credits expire 60 days after earning
            <br />
            • You can redeem expiring credits for free items from our marketplace
            <br />
            • Redeemed items will be shipped to your registered address
          </Typography>
        </Alert>
      </Stack>

      {/* Redemption dialog */}
      <Dialog
        open={showRedemptionDialog}
        onClose={() => setShowRedemptionDialog(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          Redeem Expiring Credits
        </DialogTitle>
        
        <DialogContent>
          <Stack spacing={3}>
            <Alert severity="warning">
              <Typography variant="body2">
                You are about to redeem expiring credits for free items. 
                This action cannot be undone.
              </Typography>
            </Alert>

            <Typography variant="h6" gutterBottom>
              Credits to Redeem:
            </Typography>
            
            <List>
              {selectedCredits.map((credit) => (
                <ListItem key={credit.id}>
                  <ListItemIcon>
                    <LocalOffer color="primary" />
                  </ListItemIcon>
                  <ListItemText
                    primary={`$${credit.amount} ${credit.type} credit`}
                    secondary={
                      credit.type === 'item_specific' && credit.itemName
                        ? `Specific to: ${credit.itemName}`
                        : 'General credit - can be used anywhere'
                    }
                  />
                </ListItem>
              ))}
            </List>

            <Card variant="outlined">
              <CardContent>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Typography variant="h6">
                    Total Redemption Value:
                  </Typography>
                  <Typography variant="h5" fontWeight={600} color="primary">
                    ${selectedCredits.reduce((sum, credit) => sum + credit.amount, 0)}
                  </Typography>
                </Stack>
              </CardContent>
            </Card>

            <Typography variant="body2" color="text.secondary">
              These credits will be converted to free items that will be shipped to your address. 
              You will receive an email confirmation with tracking information.
            </Typography>
          </Stack>
        </DialogContent>
        
        <DialogActions>
          <Button onClick={() => setShowRedemptionDialog(false)}>
            Cancel
          </Button>
          <Button
            onClick={confirmRedemption}
            variant="contained"
            sx={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              '&:hover': {
                background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
              },
            }}
          >
            Confirm Redemption
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default ExpiringCredits;