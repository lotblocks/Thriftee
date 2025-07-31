import React, { useState } from 'react';
import {
  Box,
  Stack,
  Typography,
  Button,
  Card,
  CardContent,
  Grid,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  InputAdornment,
  Alert,
  Divider,
} from '@mui/material';
import {
  AccountBalanceWallet,
  Add,
  LocalOffer,
  Star,
  Close,
} from '@mui/icons-material';
import { useNavigate } from 'react-router-dom';
import { toast } from 'react-toastify';

import PaymentForm from './PaymentForm';
import { useAppSelector, useAppDispatch } from '../../store';
import { addCredit } from '../../store/slices/creditSlice';

interface CreditPackage {
  id: string;
  amount: number;
  price: number;
  bonus: number;
  popular: boolean;
  description: string;
}

const CreditPurchase: React.FC = () => {
  const navigate = useNavigate();
  const dispatch = useAppDispatch();
  const { balance } = useAppSelector(state => state.credit);
  const [selectedPackage, setSelectedPackage] = useState<CreditPackage | null>(null);
  const [customAmount, setCustomAmount] = useState('');
  const [showPaymentDialog, setShowPaymentDialog] = useState(false);
  const [showCustomDialog, setShowCustomDialog] = useState(false);

  // Predefined credit packages
  const creditPackages: CreditPackage[] = [
    {
      id: 'starter',
      amount: 25,
      price: 25,
      bonus: 0,
      popular: false,
      description: 'Perfect for trying out a few raffles',
    },
    {
      id: 'popular',
      amount: 50,
      price: 50,
      bonus: 5,
      popular: true,
      description: 'Most popular choice with bonus credits',
    },
    {
      id: 'value',
      amount: 100,
      price: 100,
      bonus: 15,
      popular: false,
      description: 'Great value with extra bonus credits',
    },
    {
      id: 'premium',
      amount: 200,
      price: 200,
      bonus: 40,
      popular: false,
      description: 'Maximum value for serious participants',
    },
  ];

  const handlePackageSelect = (pkg: CreditPackage) => {
    setSelectedPackage(pkg);
    setShowPaymentDialog(true);
  };

  const handleCustomPurchase = () => {
    const amount = parseFloat(customAmount);
    if (amount < 10) {
      toast.error('Minimum purchase amount is $10');
      return;
    }
    if (amount > 1000) {
      toast.error('Maximum purchase amount is $1000');
      return;
    }

    const customPackage: CreditPackage = {
      id: 'custom',
      amount,
      price: amount,
      bonus: amount >= 100 ? Math.floor(amount * 0.1) : 0,
      popular: false,
      description: 'Custom credit amount',
    };

    setSelectedPackage(customPackage);
    setShowCustomDialog(false);
    setShowPaymentDialog(true);
  };

  const handlePaymentSuccess = (paymentIntent: any) => {
    if (selectedPackage) {
      // Add credits to user's balance
      dispatch(addCredit({
        id: Date.now().toString(),
        userId: 'current-user',
        amount: selectedPackage.amount + selectedPackage.bonus,
        type: 'general',
        createdAt: new Date().toISOString(),
      }));

      toast.success(
        `Successfully purchased $${selectedPackage.amount + selectedPackage.bonus} in credits!`
      );
      
      setShowPaymentDialog(false);
      setSelectedPackage(null);
      
      // Navigate to dashboard or credits page
      navigate('/dashboard');
    }
  };

  const handlePaymentError = (error: string) => {
    toast.error(`Payment failed: ${error}`);
  };

  const getTotalCredits = (pkg: CreditPackage) => pkg.amount + pkg.bonus;
  const getSavingsPercentage = (pkg: CreditPackage) => 
    pkg.bonus > 0 ? Math.round((pkg.bonus / pkg.amount) * 100) : 0;

  return (
    <Box>
      <Stack spacing={4}>
        {/* Header */}
        <Box>
          <Stack direction="row" alignItems="center" spacing={2} mb={2}>
            <AccountBalanceWallet color="primary" sx={{ fontSize: 32 }} />
            <Box>
              <Typography variant="h4" fontWeight={600}>
                Purchase Credits
              </Typography>
              <Typography variant="body1" color="text.secondary">
                Add credits to your account to participate in raffles
              </Typography>
            </Box>
          </Stack>

          {/* Current balance */}
          <Card variant="outlined" sx={{ mb: 3 }}>
            <CardContent>
              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Typography variant="body1" color="text.secondary">
                  Current Balance
                </Typography>
                <Typography variant="h5" fontWeight={600} color="primary">
                  ${balance.toFixed(2)}
                </Typography>
              </Stack>
            </CardContent>
          </Card>
        </Box>

        {/* Credit packages */}
        <Box>
          <Typography variant="h5" gutterBottom fontWeight={600}>
            Choose a Credit Package
          </Typography>
          <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
            Select from our popular packages or choose a custom amount
          </Typography>

          <Grid container spacing={3}>
            {creditPackages.map((pkg) => (
              <Grid item xs={12} sm={6} md={3} key={pkg.id}>
                <Card
                  sx={{
                    height: '100%',
                    cursor: 'pointer',
                    transition: 'all 0.2s',
                    border: pkg.popular ? 2 : 1,
                    borderColor: pkg.popular ? 'primary.main' : 'divider',
                    position: 'relative',
                    '&:hover': {
                      transform: 'translateY(-4px)',
                      boxShadow: 4,
                    },
                  }}
                  onClick={() => handlePackageSelect(pkg)}
                >
                  {pkg.popular && (
                    <Chip
                      label="Most Popular"
                      color="primary"
                      size="small"
                      icon={<Star />}
                      sx={{
                        position: 'absolute',
                        top: -10,
                        left: '50%',
                        transform: 'translateX(-50%)',
                        fontWeight: 600,
                      }}
                    />
                  )}
                  
                  <CardContent sx={{ textAlign: 'center', p: 3 }}>
                    <Typography variant="h3" fontWeight={700} color="primary" gutterBottom>
                      ${getTotalCredits(pkg)}
                    </Typography>
                    
                    <Typography variant="h6" color="text.secondary" gutterBottom>
                      Pay ${pkg.price}
                    </Typography>

                    {pkg.bonus > 0 && (
                      <Chip
                        label={`+$${pkg.bonus} Bonus (${getSavingsPercentage(pkg)}%)`}
                        color="success"
                        size="small"
                        sx={{ mb: 2 }}
                      />
                    )}

                    <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                      {pkg.description}
                    </Typography>

                    <Button
                      variant={pkg.popular ? 'contained' : 'outlined'}
                      fullWidth
                      sx={pkg.popular ? {
                        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                        '&:hover': {
                          background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                        },
                      } : {}}
                    >
                      Purchase Credits
                    </Button>
                  </CardContent>
                </Card>
              </Grid>
            ))}

            {/* Custom amount card */}
            <Grid item xs={12} sm={6} md={3}>
              <Card
                sx={{
                  height: '100%',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                  border: 1,
                  borderColor: 'divider',
                  borderStyle: 'dashed',
                  '&:hover': {
                    transform: 'translateY(-4px)',
                    boxShadow: 4,
                    borderColor: 'primary.main',
                  },
                }}
                onClick={() => setShowCustomDialog(true)}
              >
                <CardContent sx={{ textAlign: 'center', p: 3 }}>
                  <Add sx={{ fontSize: 48, color: 'primary.main', mb: 2 }} />
                  
                  <Typography variant="h6" fontWeight={600} gutterBottom>
                    Custom Amount
                  </Typography>
                  
                  <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                    Choose your own credit amount ($10 - $1000)
                  </Typography>

                  <Button variant="outlined" fullWidth>
                    Choose Amount
                  </Button>
                </CardContent>
              </Card>
            </Grid>
          </Grid>
        </Box>

        {/* Benefits section */}
        <Box>
          <Typography variant="h5" gutterBottom fontWeight={600}>
            Why Purchase Credits?
          </Typography>
          
          <Grid container spacing={3}>
            <Grid item xs={12} md={4}>
              <Card variant="outlined">
                <CardContent>
                  <LocalOffer color="primary" sx={{ fontSize: 32, mb: 2 }} />
                  <Typography variant="h6" fontWeight={600} gutterBottom>
                    No-Loss Guarantee
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    If you don't win, you get credits back equal to your purchase value. 
                    You never lose money!
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
            
            <Grid item xs={12} md={4}>
              <Card variant="outlined">
                <CardContent>
                  <Star color="primary" sx={{ fontSize: 32, mb: 2 }} />
                  <Typography variant="h6" fontWeight={600} gutterBottom>
                    Bonus Credits
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Larger purchases include bonus credits, giving you more value 
                    and more chances to win.
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
            
            <Grid item xs={12} md={4}>
              <Card variant="outlined">
                <CardContent>
                  <AccountBalanceWallet color="primary" sx={{ fontSize: 32, mb: 2 }} />
                  <Typography variant="h6" fontWeight={600} gutterBottom>
                    Flexible Usage
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    Use credits for any raffle, redeem for free items, or save them 
                    for future purchases.
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
          </Grid>
        </Box>
      </Stack>

      {/* Custom amount dialog */}
      <Dialog
        open={showCustomDialog}
        onClose={() => setShowCustomDialog(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Typography variant="h6">Custom Credit Amount</Typography>
            <Button onClick={() => setShowCustomDialog(false)}>
              <Close />
            </Button>
          </Stack>
        </DialogTitle>
        
        <DialogContent>
          <Stack spacing={3} sx={{ pt: 1 }}>
            <Typography variant="body2" color="text.secondary">
              Enter the amount of credits you'd like to purchase (minimum $10, maximum $1000).
            </Typography>
            
            <TextField
              label="Credit Amount"
              type="number"
              value={customAmount}
              onChange={(e) => setCustomAmount(e.target.value)}
              fullWidth
              InputProps={{
                startAdornment: <InputAdornment position="start">$</InputAdornment>,
              }}
              inputProps={{
                min: 10,
                max: 1000,
                step: 1,
              }}
            />

            {parseFloat(customAmount) >= 100 && (
              <Alert severity="success">
                <Typography variant="body2">
                  You'll receive a 10% bonus (${Math.floor(parseFloat(customAmount) * 0.1)}) 
                  for purchases over $100!
                </Typography>
              </Alert>
            )}
          </Stack>
        </DialogContent>
        
        <DialogActions>
          <Button onClick={() => setShowCustomDialog(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleCustomPurchase}
            variant="contained"
            disabled={!customAmount || parseFloat(customAmount) < 10 || parseFloat(customAmount) > 1000}
            sx={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              '&:hover': {
                background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
              },
            }}
          >
            Continue to Payment
          </Button>
        </DialogActions>
      </Dialog>

      {/* Payment dialog */}
      <Dialog
        open={showPaymentDialog}
        onClose={() => setShowPaymentDialog(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Typography variant="h6">Complete Your Purchase</Typography>
            <Button onClick={() => setShowPaymentDialog(false)}>
              <Close />
            </Button>
          </Stack>
        </DialogTitle>
        
        <DialogContent>
          {selectedPackage && (
            <Stack spacing={3}>
              {/* Purchase summary */}
              <Card variant="outlined">
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    Purchase Summary
                  </Typography>
                  <Stack spacing={1}>
                    <Stack direction="row" justifyContent="space-between">
                      <Typography>Credits</Typography>
                      <Typography>${selectedPackage.amount}</Typography>
                    </Stack>
                    {selectedPackage.bonus > 0 && (
                      <Stack direction="row" justifyContent="space-between">
                        <Typography color="success.main">Bonus Credits</Typography>
                        <Typography color="success.main">+${selectedPackage.bonus}</Typography>
                      </Stack>
                    )}
                    <Divider />
                    <Stack direction="row" justifyContent="space-between">
                      <Typography fontWeight={600}>Total Credits</Typography>
                      <Typography fontWeight={600} color="primary">
                        ${getTotalCredits(selectedPackage)}
                      </Typography>
                    </Stack>
                    <Stack direction="row" justifyContent="space-between">
                      <Typography fontWeight={600}>Amount to Pay</Typography>
                      <Typography fontWeight={600}>
                        ${selectedPackage.price}
                      </Typography>
                    </Stack>
                  </Stack>
                </CardContent>
              </Card>

              {/* Payment form */}
              <PaymentForm
                amount={selectedPackage.price}
                currency="USD"
                description={`Credit Purchase - $${getTotalCredits(selectedPackage)} credits`}
                onSuccess={handlePaymentSuccess}
                onError={handlePaymentError}
              />
            </Stack>
          )}
        </DialogContent>
      </Dialog>
    </Box>
  );
};

export default CreditPurchase;