import React, { useState } from 'react';
import {
  Box,
  Stack,
  Typography,
  Button,
  Chip,
  IconButton,
  Alert,
  CircularProgress,
  Divider,
  Card,
  CardContent,
  Collapse,
} from '@mui/material';
import {
  ShoppingCart,
  Clear,
  Close,
  ExpandMore,
  ExpandLess,
  AccountBalanceWallet,
  CreditCard,
  Info,
} from '@mui/icons-material';

import { useAppSelector } from '../../store';

interface MobileBoxSelectorProps {
  selectedBoxes: number[];
  totalCost: number;
  currency: string;
  onPurchase: () => void;
  onClearSelection: () => void;
  onClose: () => void;
  isLoading: boolean;
  disabled: boolean;
}

const MobileBoxSelector: React.FC<MobileBoxSelectorProps> = ({
  selectedBoxes,
  totalCost,
  currency,
  onPurchase,
  onClearSelection,
  onClose,
  isLoading,
  disabled,
}) => {
  const { balance } = useAppSelector(state => state.credit);
  const [expanded, setExpanded] = useState(false);
  const [paymentMethod, setPaymentMethod] = useState<'credits' | 'card'>('credits');

  const hasSelection = selectedBoxes.length > 0;
  const canAffordWithCredits = balance >= totalCost;
  const suggestedPaymentMethod = canAffordWithCredits ? 'credits' : 'card';

  // Sort selected boxes for display
  const sortedBoxes = [...selectedBoxes].sort((a, b) => a - b);

  // Group consecutive boxes for better display
  const groupConsecutiveBoxes = (boxes: number[]): string => {
    if (boxes.length === 0) return '';
    if (boxes.length === 1) return boxes[0].toString();

    const groups: string[] = [];
    let start = boxes[0];
    let end = boxes[0];

    for (let i = 1; i < boxes.length; i++) {
      if (boxes[i] === end + 1) {
        end = boxes[i];
      } else {
        if (start === end) {
          groups.push(start.toString());
        } else if (end === start + 1) {
          groups.push(`${start}, ${end}`);
        } else {
          groups.push(`${start}-${end}`);
        }
        start = boxes[i];
        end = boxes[i];
      }
    }

    // Add the last group
    if (start === end) {
      groups.push(start.toString());
    } else if (end === start + 1) {
      groups.push(`${start}, ${end}`);
    } else {
      groups.push(`${start}-${end}`);
    }

    return groups.join(', ');
  };

  const handlePaymentMethodChange = (method: 'credits' | 'card') => {
    setPaymentMethod(method);
  };

  if (!hasSelection) {
    return (
      <Box sx={{ p: 3, textAlign: 'center' }}>
        <ShoppingCart sx={{ fontSize: 48, color: 'text.secondary', mb: 2 }} />
        <Typography variant="h6" gutterBottom>
          No Boxes Selected
        </Typography>
        <Typography color="text.secondary" sx={{ mb: 3 }}>
          Select boxes from the grid to see your purchase summary here
        </Typography>
        <Button variant="outlined" onClick={onClose} fullWidth>
          Close
        </Button>
      </Box>
    );
  }

  return (
    <Box sx={{ maxHeight: '80vh', overflow: 'auto' }}>
      {/* Drag handle */}
      <Box
        sx={{
          width: 40,
          height: 4,
          backgroundColor: 'grey.300',
          borderRadius: 2,
          mx: 'auto',
          my: 1,
        }}
      />

      <Box sx={{ p: 3 }}>
        <Stack spacing={3}>
          {/* Header */}
          <Stack direction="row" justifyContent="space-between" alignItems="center">
            <Stack direction="row" alignItems="center" spacing={2}>
              <ShoppingCart color="primary" />
              <Box>
                <Typography variant="h6" fontWeight={600}>
                  {selectedBoxes.length} Box{selectedBoxes.length !== 1 ? 'es' : ''} Selected
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Ready for purchase
                </Typography>
              </Box>
            </Stack>
            <IconButton onClick={onClose} size="small">
              <Close />
            </IconButton>
          </Stack>

          {/* Selected boxes summary */}
          <Card variant="outlined">
            <CardContent sx={{ p: 2 }}>
              <Stack direction="row" justifyContent="space-between" alignItems="center" mb={1}>
                <Typography variant="body2" fontWeight={500}>
                  Selected boxes:
                </Typography>
                <Button
                  size="small"
                  endIcon={expanded ? <ExpandLess /> : <ExpandMore />}
                  onClick={() => setExpanded(!expanded)}
                  sx={{ minWidth: 'auto' }}
                >
                  {expanded ? 'Hide' : 'Show'}
                </Button>
              </Stack>
              
              <Collapse in={expanded}>
                <Box sx={{ p: 1, backgroundColor: 'grey.50', borderRadius: 1, mb: 2 }}>
                  <Typography variant="body2">
                    Boxes: {groupConsecutiveBoxes(sortedBoxes)}
                  </Typography>
                </Box>
              </Collapse>

              {!expanded && (
                <Typography variant="body2" color="text.secondary">
                  {groupConsecutiveBoxes(sortedBoxes.slice(0, 8))}
                  {sortedBoxes.length > 8 && ` ... +${sortedBoxes.length - 8} more`}
                </Typography>
              )}
            </CardContent>
          </Card>

          {/* Cost breakdown */}
          <Card variant="outlined">
            <CardContent sx={{ p: 2 }}>
              <Stack spacing={2}>
                <Stack direction="row" justifyContent="space-between" alignItems="center">
                  <Typography variant="body1">
                    {selectedBoxes.length} × ${(totalCost / selectedBoxes.length).toFixed(2)}
                  </Typography>
                  <Typography variant="h6" fontWeight={600} color="primary">
                    ${totalCost.toFixed(2)} {currency}
                  </Typography>
                </Stack>
              </Stack>
            </CardContent>
          </Card>

          {/* Payment method selection */}
          <Box>
            <Typography variant="subtitle2" gutterBottom fontWeight={600}>
              Payment Method:
            </Typography>
            <Stack spacing={1}>
              <Button
                variant={paymentMethod === 'credits' ? 'contained' : 'outlined'}
                onClick={() => handlePaymentMethodChange('credits')}
                disabled={!canAffordWithCredits}
                startIcon={<AccountBalanceWallet />}
                fullWidth
                sx={{
                  justifyContent: 'flex-start',
                  textAlign: 'left',
                  p: 2,
                }}
              >
                <Box sx={{ flex: 1, textAlign: 'left' }}>
                  <Typography variant="body2" fontWeight={500}>
                    Credits
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    ${balance.toFixed(2)} available
                  </Typography>
                </Box>
              </Button>
              
              <Button
                variant={paymentMethod === 'card' ? 'contained' : 'outlined'}
                onClick={() => handlePaymentMethodChange('card')}
                startIcon={<CreditCard />}
                fullWidth
                sx={{
                  justifyContent: 'flex-start',
                  textAlign: 'left',
                  p: 2,
                }}
              >
                <Box sx={{ flex: 1, textAlign: 'left' }}>
                  <Typography variant="body2" fontWeight={500}>
                    Credit Card
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    Secure payment processing
                  </Typography>
                </Box>
              </Button>
            </Stack>
          </Box>

          {/* Warnings and info */}
          {paymentMethod === 'credits' && !canAffordWithCredits && (
            <Alert severity="warning" icon={<Info />}>
              <Typography variant="body2">
                Insufficient credits. You need ${(totalCost - balance).toFixed(2)} more.
              </Typography>
            </Alert>
          )}

          {paymentMethod === 'credits' && canAffordWithCredits && (
            <Alert severity="info" icon={<Info />}>
              <Typography variant="body2">
                After purchase: ${(balance - totalCost).toFixed(2)} credits remaining
              </Typography>
            </Alert>
          )}

          <Divider />

          {/* Action buttons */}
          <Stack spacing={2}>
            <Button
              variant="contained"
              size="large"
              onClick={onPurchase}
              disabled={disabled || isLoading || (paymentMethod === 'credits' && !canAffordWithCredits)}
              startIcon={
                isLoading ? (
                  <CircularProgress size={20} color="inherit" />
                ) : (
                  <ShoppingCart />
                )
              }
              fullWidth
              sx={{
                py: 1.5,
                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                '&:hover': {
                  background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                },
                fontSize: '1rem',
                fontWeight: 600,
              }}
            >
              {isLoading
                ? 'Processing...'
                : `Purchase ${selectedBoxes.length} Box${selectedBoxes.length !== 1 ? 'es' : ''} - $${totalCost.toFixed(2)}`
              }
            </Button>

            <Stack direction="row" spacing={2}>
              <Button
                variant="outlined"
                onClick={onClearSelection}
                startIcon={<Clear />}
                fullWidth
                color="error"
              >
                Clear Selection
              </Button>
              
              <Button
                variant="outlined"
                onClick={onClose}
                fullWidth
              >
                Continue Shopping
              </Button>
            </Stack>
          </Stack>

          {/* Additional info */}
          <Typography variant="caption" color="text.secondary" textAlign="center">
            {paymentMethod === 'credits' 
              ? 'Payment will be deducted from your credit balance'
              : 'You will be redirected to secure payment processing'
            }
          </Typography>
        </Stack>
      </Box>
    </Box>
  );
};

export default MobileBoxSelector;