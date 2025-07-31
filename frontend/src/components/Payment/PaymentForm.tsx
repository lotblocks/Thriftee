import React, { useState, useEffect } from 'react';
import {
  Box,
  Stack,
  Typography,
  Button,
  Card,
  CardContent,
  Alert,
  Divider,
  Chip,
  CircularProgress,
} from '@mui/material';
import {
  CreditCard,
  Lock,
  CheckCircle,
  Error,
} from '@mui/icons-material';
import {
  useStripe,
  useElements,
  CardNumberElement,
  CardExpiryElement,
  CardCvcElement,
} from '@stripe/react-stripe-js';
import { toast } from 'react-toastify';

import { apiRequest } from '../../services/api';

interface PaymentFormProps {
  amount: number;
  currency: string;
  description: string;
  onSuccess: (paymentIntent: any) => void;
  onError: (error: string) => void;
  disabled?: boolean;
}

const PaymentForm: React.FC<PaymentFormProps> = ({
  amount,
  currency,
  description,
  onSuccess,
  onError,
  disabled = false,
}) => {
  const stripe = useStripe();
  const elements = useElements();
  const [isProcessing, setIsProcessing] = useState(false);
  const [clientSecret, setClientSecret] = useState<string | null>(null);
  const [paymentError, setPaymentError] = useState<string | null>(null);

  // Stripe Elements styling
  const elementOptions = {
    style: {
      base: {
        fontSize: '16px',
        color: '#424770',
        '::placeholder': {
          color: '#aab7c4',
        },
        fontFamily: 'Inter, system-ui, sans-serif',
      },
      invalid: {
        color: '#9e2146',
      },
    },
  };

  // Create payment intent when component mounts
  useEffect(() => {
    const createPaymentIntent = async () => {
      try {
        const response = await apiRequest.post('/payments/create-intent', {
          amount: Math.round(amount * 100), // Convert to cents
          currency: currency.toLowerCase(),
          description,
        });
        setClientSecret(response.clientSecret);
      } catch (error: any) {
        console.error('Failed to create payment intent:', error);
        onError(error.response?.data?.message || 'Failed to initialize payment');
      }
    };

    if (amount > 0) {
      createPaymentIntent();
    }
  }, [amount, currency, description, onError]);

  const handleSubmit = async (event: React.FormEvent) => {
    event.preventDefault();

    if (!stripe || !elements || !clientSecret) {
      return;
    }

    setIsProcessing(true);
    setPaymentError(null);

    const cardElement = elements.getElement(CardNumberElement);
    if (!cardElement) {
      setPaymentError('Card element not found');
      setIsProcessing(false);
      return;
    }

    try {
      const { error, paymentIntent } = await stripe.confirmCardPayment(clientSecret, {
        payment_method: {
          card: cardElement,
        },
      });

      if (error) {
        setPaymentError(error.message || 'Payment failed');
        onError(error.message || 'Payment failed');
      } else if (paymentIntent && paymentIntent.status === 'succeeded') {
        onSuccess(paymentIntent);
        toast.success('Payment successful!');
      }
    } catch (error: any) {
      console.error('Payment error:', error);
      setPaymentError('An unexpected error occurred');
      onError('An unexpected error occurred');
    } finally {
      setIsProcessing(false);
    }
  };

  if (!clientSecret) {
    return (
      <Card>
        <CardContent>
          <Box
            sx={{
              display: 'flex',
              justifyContent: 'center',
              alignItems: 'center',
              py: 4,
            }}
          >
            <CircularProgress />
          </Box>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardContent>
        <form onSubmit={handleSubmit}>
          <Stack spacing={3}>
            {/* Header */}
            <Box>
              <Stack direction="row" alignItems="center" spacing={2} mb={1}>
                <CreditCard color="primary" />
                <Typography variant="h6" fontWeight={600}>
                  Payment Details
                </Typography>
              </Stack>
              <Typography variant="body2" color="text.secondary">
                Enter your card information to complete the purchase
              </Typography>
            </Box>

            {/* Payment summary */}
            <Box
              sx={{
                p: 2,
                backgroundColor: 'grey.50',
                borderRadius: 1,
                border: 1,
                borderColor: 'grey.200',
              }}
            >
              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Typography variant="body1" fontWeight={500}>
                  {description}
                </Typography>
                <Typography variant="h6" fontWeight={600} color="primary">
                  ${amount.toFixed(2)} {currency.toUpperCase()}
                </Typography>
              </Stack>
            </Box>

            {/* Card number */}
            <Box>
              <Typography variant="body2" fontWeight={500} gutterBottom>
                Card Number
              </Typography>
              <Box
                sx={{
                  p: 2,
                  border: 1,
                  borderColor: 'grey.300',
                  borderRadius: 1,
                  '&:focus-within': {
                    borderColor: 'primary.main',
                    borderWidth: 2,
                  },
                }}
              >
                <CardNumberElement options={elementOptions} />
              </Box>
            </Box>

            {/* Expiry and CVC */}
            <Stack direction="row" spacing={2}>
              <Box sx={{ flex: 1 }}>
                <Typography variant="body2" fontWeight={500} gutterBottom>
                  Expiry Date
                </Typography>
                <Box
                  sx={{
                    p: 2,
                    border: 1,
                    borderColor: 'grey.300',
                    borderRadius: 1,
                    '&:focus-within': {
                      borderColor: 'primary.main',
                      borderWidth: 2,
                    },
                  }}
                >
                  <CardExpiryElement options={elementOptions} />
                </Box>
              </Box>
              
              <Box sx={{ flex: 1 }}>
                <Typography variant="body2" fontWeight={500} gutterBottom>
                  CVC
                </Typography>
                <Box
                  sx={{
                    p: 2,
                    border: 1,
                    borderColor: 'grey.300',
                    borderRadius: 1,
                    '&:focus-within': {
                      borderColor: 'primary.main',
                      borderWidth: 2,
                    },
                  }}
                >
                  <CardCvcElement options={elementOptions} />
                </Box>
              </Box>
            </Stack>

            {/* Error display */}
            {paymentError && (
              <Alert severity="error" icon={<Error />}>
                {paymentError}
              </Alert>
            )}

            {/* Security notice */}
            <Box
              sx={{
                p: 2,
                backgroundColor: 'success.light',
                borderRadius: 1,
                border: 1,
                borderColor: 'success.main',
              }}
            >
              <Stack direction="row" alignItems="center" spacing={2}>
                <Lock color="success" />
                <Box>
                  <Typography variant="body2" fontWeight={500} color="success.dark">
                    Secure Payment
                  </Typography>
                  <Typography variant="caption" color="success.dark">
                    Your payment information is encrypted and secure. We never store your card details.
                  </Typography>
                </Box>
              </Stack>
            </Box>

            <Divider />

            {/* Submit button */}
            <Button
              type="submit"
              variant="contained"
              size="large"
              disabled={!stripe || isProcessing || disabled}
              startIcon={
                isProcessing ? (
                  <CircularProgress size={20} color="inherit" />
                ) : (
                  <CreditCard />
                )
              }
              sx={{
                py: 1.5,
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                '&:hover': {
                  background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                },
                '&:disabled': {
                  background: 'grey.300',
                },
              }}
            >
              {isProcessing
                ? 'Processing Payment...'
                : `Pay $${amount.toFixed(2)} ${currency.toUpperCase()}`
              }
            </Button>

            {/* Accepted cards */}
            <Box sx={{ textAlign: 'center' }}>
              <Typography variant="caption" color="text.secondary" gutterBottom>
                We accept
              </Typography>
              <Stack direction="row" justifyContent="center" spacing={1} mt={1}>
                {['Visa', 'Mastercard', 'American Express', 'Discover'].map((card) => (
                  <Chip
                    key={card}
                    label={card}
                    size="small"
                    variant="outlined"
                    sx={{ fontSize: '0.7rem' }}
                  />
                ))}
              </Stack>
            </Box>
          </Stack>
        </form>
      </CardContent>
    </Card>
  );
};

export default PaymentForm;