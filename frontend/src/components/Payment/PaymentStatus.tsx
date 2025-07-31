import React from 'react';
import {
  Box,
  Stack,
  Typography,
  Card,
  CardContent,
  Button,
  Alert,
  Stepper,
  Step,
  StepLabel,
  StepContent,
  Chip,
  Divider,
} from '@mui/material';
import {
  CheckCircle,
  Error,
  Schedule,
  Refresh,
  Receipt,
  Home,
  CreditCard,
} from '@mui/icons-material';
import { useNavigate } from 'react-router-dom';

import { Payment, PaymentStatus as PaymentStatusType } from '../../types/payment';

interface PaymentStatusProps {
  payment: Payment;
  onRetry?: () => void;
  onDownloadReceipt?: () => void;
}

const PaymentStatus: React.FC<PaymentStatusProps> = ({
  payment,
  onRetry,
  onDownloadReceipt,
}) => {
  const navigate = useNavigate();

  const getStatusConfig = (status: PaymentStatusType) => {
    switch (status) {
      case 'completed':
        return {
          icon: <CheckCircle color="success" sx={{ fontSize: 64 }} />,
          title: 'Payment Successful!',
          message: 'Your payment has been processed successfully.',
          color: 'success' as const,
          severity: 'success' as const,
        };
      case 'pending':
        return {
          icon: <Schedule color="warning" sx={{ fontSize: 64 }} />,
          title: 'Payment Processing',
          message: 'Your payment is being processed. This may take a few minutes.',
          color: 'warning' as const,
          severity: 'info' as const,
        };
      case 'failed':
        return {
          icon: <Error color="error" sx={{ fontSize: 64 }} />,
          title: 'Payment Failed',
          message: 'Your payment could not be processed. Please try again.',
          color: 'error' as const,
          severity: 'error' as const,
        };
      case 'refunded':
        return {
          icon: <Refresh color="info" sx={{ fontSize: 64 }} />,
          title: 'Payment Refunded',
          message: 'Your payment has been refunded successfully.',
          color: 'info' as const,
          severity: 'info' as const,
        };
      default:
        return {
          icon: <Schedule color="default" sx={{ fontSize: 64 }} />,
          title: 'Payment Status Unknown',
          message: 'We are checking the status of your payment.',
          color: 'default' as const,
          severity: 'info' as const,
        };
    }
  };

  const statusConfig = getStatusConfig(payment.status);

  // Payment processing steps
  const getPaymentSteps = () => {
    const steps = [
      {
        label: 'Payment Initiated',
        description: 'Payment request created',
        completed: true,
        timestamp: payment.createdAt,
      },
      {
        label: 'Processing Payment',
        description: 'Verifying payment details',
        completed: payment.status !== 'pending',
        timestamp: payment.updatedAt,
      },
      {
        label: 'Payment Complete',
        description: payment.status === 'completed' 
          ? 'Payment processed successfully'
          : payment.status === 'failed'
          ? 'Payment processing failed'
          : payment.status === 'refunded'
          ? 'Payment refunded'
          : 'Awaiting completion',
        completed: payment.status === 'completed' || payment.status === 'refunded',
        timestamp: payment.completedAt,
      },
    ];

    if (payment.status === 'failed') {
      steps[2].completed = false;
      steps[2].description = 'Payment processing failed';
    }

    return steps;
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getNextSteps = () => {
    switch (payment.status) {
      case 'completed':
        if (payment.type === 'credit_purchase') {
          return [
            'Your credits have been added to your account',
            'You can now use them to participate in raffles',
            'Check your dashboard for updated balance',
          ];
        } else if (payment.type === 'box_purchase') {
          return [
            'Your raffle boxes have been reserved',
            'You will be notified when the raffle is complete',
            'Check the raffle page for updates',
          ];
        }
        return ['Your payment has been processed successfully'];
      
      case 'pending':
        return [
          'Please wait while we process your payment',
          'This usually takes 1-3 minutes',
          'You will receive a confirmation email once complete',
        ];
      
      case 'failed':
        return [
          'Please check your payment method details',
          'Ensure you have sufficient funds',
          'Try using a different payment method',
          'Contact support if the issue persists',
        ];
      
      case 'refunded':
        return [
          'The refund has been processed',
          'It may take 3-5 business days to appear in your account',
          'You will receive a confirmation email',
        ];
      
      default:
        return ['We are checking the status of your payment'];
    }
  };

  return (
    <Box>
      <Stack spacing={4}>
        {/* Status header */}
        <Card>
          <CardContent>
            <Stack alignItems="center" spacing={3} sx={{ textAlign: 'center', py: 4 }}>
              {statusConfig.icon}
              
              <Box>
                <Typography variant="h4" fontWeight={600} gutterBottom>
                  {statusConfig.title}
                </Typography>
                <Typography variant="body1" color="text.secondary">
                  {statusConfig.message}
                </Typography>
              </Box>

              <Chip
                label={payment.status.charAt(0).toUpperCase() + payment.status.slice(1)}
                color={statusConfig.color}
                size="large"
                sx={{ px: 2, py: 1, fontSize: '1rem' }}
              />
            </Stack>
          </CardContent>
        </Card>

        {/* Payment details */}
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom fontWeight={600}>
              Payment Details
            </Typography>
            
            <Stack spacing={2}>
              <Stack direction="row" justifyContent="space-between">
                <Typography color="text.secondary">Payment ID</Typography>
                <Typography fontFamily="monospace">{payment.id}</Typography>
              </Stack>
              
              <Stack direction="row" justifyContent="space-between">
                <Typography color="text.secondary">Amount</Typography>
                <Typography fontWeight={600}>
                  ${payment.amount.toFixed(2)} {payment.currency}
                </Typography>
              </Stack>
              
              <Stack direction="row" justifyContent="space-between">
                <Typography color="text.secondary">Description</Typography>
                <Typography>{payment.description}</Typography>
              </Stack>
              
              <Stack direction="row" justifyContent="space-between">
                <Typography color="text.secondary">Date</Typography>
                <Typography>{formatDate(payment.createdAt)}</Typography>
              </Stack>

              {payment.stripePaymentIntentId && (
                <Stack direction="row" justifyContent="space-between">
                  <Typography color="text.secondary">Stripe ID</Typography>
                  <Typography fontFamily="monospace" fontSize="0.875rem">
                    {payment.stripePaymentIntentId}
                  </Typography>
                </Stack>
              )}
            </Stack>
          </CardContent>
        </Card>

        {/* Payment progress */}
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom fontWeight={600}>
              Payment Progress
            </Typography>
            
            <Stepper orientation="vertical">
              {getPaymentSteps().map((step, index) => (
                <Step key={index} active={step.completed}>
                  <StepLabel>
                    <Box>
                      <Typography variant="body1" fontWeight={500}>
                        {step.label}
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        {step.description}
                      </Typography>
                      {step.timestamp && (
                        <Typography variant="caption" color="text.secondary">
                          {formatDate(step.timestamp)}
                        </Typography>
                      )}
                    </Box>
                  </StepLabel>
                </Step>
              ))}
            </Stepper>
          </CardContent>
        </Card>

        {/* Next steps */}
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom fontWeight={600}>
              What's Next?
            </Typography>
            
            <Alert severity={statusConfig.severity} sx={{ mb: 3 }}>
              <Stack spacing={1}>
                {getNextSteps().map((step, index) => (
                  <Typography key={index} variant="body2">
                    • {step}
                  </Typography>
                ))}
              </Stack>
            </Alert>

            {/* Action buttons */}
            <Stack direction="row" spacing={2} flexWrap="wrap">
              {payment.status === 'completed' && (
                <>
                  <Button
                    variant="contained"
                    startIcon={<Home />}
                    onClick={() => navigate('/dashboard')}
                    sx={{
                      background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                      '&:hover': {
                        background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                      },
                    }}
                  >
                    Go to Dashboard
                  </Button>
                  
                  {onDownloadReceipt && (
                    <Button
                      variant="outlined"
                      startIcon={<Receipt />}
                      onClick={onDownloadReceipt}
                    >
                      Download Receipt
                    </Button>
                  )}
                </>
              )}

              {payment.status === 'failed' && onRetry && (
                <Button
                  variant="contained"
                  startIcon={<CreditCard />}
                  onClick={onRetry}
                  color="error"
                >
                  Try Again
                </Button>
              )}

              {payment.status === 'pending' && (
                <Button
                  variant="outlined"
                  startIcon={<Refresh />}
                  onClick={() => window.location.reload()}
                >
                  Refresh Status
                </Button>
              )}

              <Button
                variant="outlined"
                onClick={() => navigate('/payments')}
              >
                View Payment History
              </Button>
            </Stack>
          </CardContent>
        </Card>

        {/* Support section */}
        {(payment.status === 'failed' || payment.status === 'pending') && (
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom fontWeight={600}>
                Need Help?
              </Typography>
              
              <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                If you're experiencing issues with your payment, our support team is here to help.
              </Typography>
              
              <Stack direction="row" spacing={2}>
                <Button variant="outlined" onClick={() => navigate('/support')}>
                  Contact Support
                </Button>
                <Button variant="text" onClick={() => navigate('/help/payments')}>
                  Payment FAQ
                </Button>
              </Stack>
            </CardContent>
          </Card>
        )}
      </Stack>
    </Box>
  );
};

export default PaymentStatus;