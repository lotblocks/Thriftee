import React, { useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  Button,
  Chip,
  Stack,
  Divider,
  IconButton,
  Collapse,
  Alert,
  CircularProgress,
} from '@mui/material';
import {
  ShoppingCart,
  Clear,
  ExpandMore,
  ExpandLess,
  AccountBalanceWallet,
  CreditCard,
  Info,
} from '@mui/icons-material';

import { useAppSelector } from '../../store';

interface BoxSelectorProps {
  selectedBoxes: number[];
  totalCost: number;
  currency: string;
  onPurchase: () => void;
  onClearSelection: () => void;
  isLoading: boolean;
  disabled: boolean;
}

const BoxSelector: React.FC<BoxSelectorProps> = ({
  selectedBoxes,
  totalCost,
  currency,
  onPurchase,
  onClearSelection,
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
      <Paper sx={{ p: 2, backgroundColor: 'grey.50' }}>
        <Stack direction="row" alignItems="center" spacing={2}>
          <ShoppingCart sx={{ color: 'text.secondary' }} />
          <Typography color="text.secondary">
            Select boxes to purchase and see your total here
          </Typography>
        </Stack>
      </Paper>
    );
  }

  return (
    <Paper sx={{ p: 2, border: 2, borderColor: 'primary.main' }}>
      <Stack spacing={2}>
        {/* Header */}
        <Stack direction="row" justifyContent="space-between" alignItems="center">
          <Stack direction="row" alignItems="center" spacing={2}>
            <ShoppingCart color="primary" />
            <Typography variant="h6" fontWeight={600}>
              {selectedBoxes.length} Box{selectedBoxes.length !== 1 ? 'es' : ''} Selected
            </Typography>
          </Stack>
          <IconButton onClick={onClearSelection} size="small">
            <Clear />
          </IconButton>
        </Stack>

        {/* Selected boxes summary */}
        <Box>
          <Stack direction="row" justifyContent="space-between" alignItems="center" mb={1}>
            <Typography variant="body2" color="text.secondary">
              Selected boxes:
            </Typography>
            <Button
              size="small"
              endIcon={expanded ? <ExpandLess /> : <ExpandMore />}
              onClick={() => setExpanded(!expanded)}
            >
              {expanded ? 'Hide' : 'Show'} Details
            </Button>
          </Stack>
          
          <Collapse in={expanded}>
            <Box sx={{ p: 2, backgroundColor: 'grey.50', borderRadius: 1, mb: 2 }}>
              <Typography variant="body2">
                Boxes: {groupConsecutiveBoxes(sortedBoxes)}
              </Typography>
            </Box>
          </Collapse>

          {!expanded && (
            <Typography variant="body2" sx={{ mb: 2 }}>
              {groupConsecutiveBoxes(sortedBoxes.slice(0, 10))}
              {sortedBoxes.length > 10 && ` ... and ${sortedBoxes.length - 10} more`}
            </Typography>
          )}
        </Box>

        <Divider />

        {/* Cost breakdown */}
        <Stack spacing={1}>
          <Stack direction="row" justifyContent="space-between">
            <Typography>
              {selectedBoxes.length} × ${totalCost / selectedBoxes.length} per box
            </Typography>
            <Typography fontWeight={600}>
              ${totalCost.toFixed(2)} {currency}
            </Typography>
          </Stack>
        </Stack>

        {/* Payment method selection */}
        <Box>
          <Typography variant="subtitle2" gutterBottom>
            Payment Method:
          </Typography>
          <Stack direction="row" spacing={1}>
            <Chip
              icon={<AccountBalanceWallet />}
              label={`Credits ($${balance.toFixed(2)} available)`}
              onClick={() => handlePaymentMethodChange('credits')}
              color={paymentMethod === 'credits' ? 'primary' : 'default'}
              variant={paymentMethod === 'credits' ? 'filled' : 'outlined'}
              disabled={!canAffordWithCredits}
            />
            <Chip
              icon={<CreditCard />}
              label="Credit Card"
              onClick={() => handlePaymentMethodChange('card')}
              color={paymentMethod === 'card' ? 'primary' : 'default'}
              variant={paymentMethod === 'card' ? 'filled' : 'outlined'}
            />
          </Stack>
        </Box>

        {/* Warnings and info */}
        {paymentMethod === 'credits' && !canAffordWithCredits && (
          <Alert severity="warning" icon={<Info />}>
            Insufficient credits. You need ${(totalCost - balance).toFixed(2)} more.
            Consider using a credit card or purchasing more credits.
          </Alert>
        )}

        {paymentMethod === 'credits' && canAffordWithCredits && (
          <Alert severity="info" icon={<Info />}>
            After purchase, you'll have ${(balance - totalCost).toFixed(2)} credits remaining.
          </Alert>
        )}

        {/* Purchase button */}
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
          sx={{
            py: 1.5,
            background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
            '&:hover': {
              background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
            },
          }}
        >
          {isLoading
            ? 'Processing...'
            : `Purchase ${selectedBoxes.length} Box${selectedBoxes.length !== 1 ? 'es' : ''} - $${totalCost.toFixed(2)}`
          }
        </Button>

        {/* Additional info */}
        <Typography variant="caption" color="text.secondary" textAlign="center">
          {paymentMethod === 'credits' 
            ? 'Payment will be deducted from your credit balance'
            : 'You will be redirected to secure payment processing'
          }
        </Typography>
      </Stack>
    </Paper>
  );
};

export default BoxSelector;