import React, { useState, useEffect } from 'react';
import {
  Box,
  Stack,
  Typography,
  Card,
  CardContent,
  Chip,
  Button,
  IconButton,
  Menu,
  MenuItem,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TablePagination,
  TextField,
  InputAdornment,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Alert,
} from '@mui/material';
import {
  History,
  FilterList,
  Search,
  Download,
  Visibility,
  Receipt,
  CheckCircle,
  Schedule,
  Error,
  Refresh,
} from '@mui/icons-material';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDateFns } from '@mui/x-date-pickers/AdapterDateFns';

import { Payment, PaymentStatus, PaymentType } from '../../types/payment';
import LoadingSpinner from '../UI/LoadingSpinner';

const PaymentHistory: React.FC = () => {
  const [payments, setPayments] = useState<Payment[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(10);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterAnchor, setFilterAnchor] = useState<null | HTMLElement>(null);
  const [selectedPayment, setSelectedPayment] = useState<Payment | null>(null);
  const [showDetailsDialog, setShowDetailsDialog] = useState(false);
  const [filters, setFilters] = useState({
    status: 'all' as PaymentStatus | 'all',
    type: 'all' as PaymentType | 'all',
    dateFrom: null as Date | null,
    dateTo: null as Date | null,
  });

  // Mock payment data - in real app, this would come from API
  useEffect(() => {
    const loadPayments = async () => {
      setIsLoading(true);
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      const mockPayments: Payment[] = [
        {
          id: 'pay_1',
          userId: 'user_1',
          amount: 100,
          currency: 'USD',
          status: 'completed',
          type: 'credit_purchase',
          stripePaymentIntentId: 'pi_1234567890',
          description: 'Credit Purchase - $115 credits',
          createdAt: '2024-01-18T10:30:00Z',
          updatedAt: '2024-01-18T10:31:00Z',
          completedAt: '2024-01-18T10:31:00Z',
        },
        {
          id: 'pay_2',
          userId: 'user_1',
          amount: 60,
          currency: 'USD',
          status: 'completed',
          type: 'box_purchase',
          raffleId: 'raffle_1',
          description: 'iPhone 15 Pro Max raffle - 5 boxes',
          createdAt: '2024-01-17T15:45:00Z',
          updatedAt: '2024-01-17T15:46:00Z',
          completedAt: '2024-01-17T15:46:00Z',
        },
        {
          id: 'pay_3',
          userId: 'user_1',
          amount: 50,
          currency: 'USD',
          status: 'completed',
          type: 'credit_purchase',
          stripePaymentIntentId: 'pi_0987654321',
          description: 'Credit Purchase - $55 credits',
          createdAt: '2024-01-15T09:20:00Z',
          updatedAt: '2024-01-15T09:21:00Z',
          completedAt: '2024-01-15T09:21:00Z',
        },
        {
          id: 'pay_4',
          userId: 'user_1',
          amount: 25,
          currency: 'USD',
          status: 'failed',
          type: 'credit_purchase',
          stripePaymentIntentId: 'pi_1122334455',
          description: 'Credit Purchase - $25 credits',
          createdAt: '2024-01-14T14:10:00Z',
          updatedAt: '2024-01-14T14:11:00Z',
        },
        {
          id: 'pay_5',
          userId: 'user_1',
          amount: 45,
          currency: 'USD',
          status: 'refunded',
          type: 'refund',
          raffleId: 'raffle_2',
          description: 'MacBook Pro raffle - refund',
          createdAt: '2024-01-13T11:00:00Z',
          updatedAt: '2024-01-13T11:30:00Z',
          completedAt: '2024-01-13T11:30:00Z',
        },
      ];
      
      setPayments(mockPayments);
      setIsLoading(false);
    };

    loadPayments();
  }, []);

  // Filter payments based on search and filters
  const filteredPayments = payments.filter(payment => {
    const matchesSearch = payment.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         payment.id.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesStatus = filters.status === 'all' || payment.status === filters.status;
    const matchesType = filters.type === 'all' || payment.type === filters.type;
    
    const paymentDate = new Date(payment.createdAt);
    const matchesDateFrom = !filters.dateFrom || paymentDate >= filters.dateFrom;
    const matchesDateTo = !filters.dateTo || paymentDate <= filters.dateTo;
    
    return matchesSearch && matchesStatus && matchesType && matchesDateFrom && matchesDateTo;
  });

  const getStatusIcon = (status: PaymentStatus) => {
    switch (status) {
      case 'completed': return <CheckCircle color="success" />;
      case 'pending': return <Schedule color="warning" />;
      case 'failed': return <Error color="error" />;
      case 'refunded': return <Refresh color="info" />;
      default: return <Schedule />;
    }
  };

  const getStatusColor = (status: PaymentStatus) => {
    switch (status) {
      case 'completed': return 'success';
      case 'pending': return 'warning';
      case 'failed': return 'error';
      case 'refunded': return 'info';
      default: return 'default';
    }
  };

  const getTypeLabel = (type: PaymentType) => {
    switch (type) {
      case 'credit_purchase': return 'Credit Purchase';
      case 'box_purchase': return 'Box Purchase';
      case 'refund': return 'Refund';
      case 'subscription': return 'Subscription';
      case 'withdrawal': return 'Withdrawal';
      default: return type;
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const handleViewDetails = (payment: Payment) => {
    setSelectedPayment(payment);
    setShowDetailsDialog(true);
  };

  const handleExportData = () => {
    // Create CSV data
    const csvData = filteredPayments.map(payment => ({
      ID: payment.id,
      Date: formatDate(payment.createdAt),
      Description: payment.description,
      Type: getTypeLabel(payment.type),
      Amount: `$${payment.amount.toFixed(2)}`,
      Status: payment.status,
    }));

    // Convert to CSV string
    const headers = Object.keys(csvData[0]).join(',');
    const rows = csvData.map(row => Object.values(row).join(',')).join('\n');
    const csvContent = `${headers}\n${rows}`;

    // Download file
    const blob = new Blob([csvContent], { type: 'text/csv' });
    const url = window.URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `payment-history-${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
    window.URL.revokeObjectURL(url);
  };

  if (isLoading) {
    return <LoadingSpinner message="Loading payment history..." />;
  }

  return (
    <Box>
      <Stack spacing={4}>
        {/* Header */}
        <Box>
          <Stack direction="row" alignItems="center" spacing={2} mb={2}>
            <History color="primary" sx={{ fontSize: 32 }} />
            <Box>
              <Typography variant="h4" fontWeight={600}>
                Payment History
              </Typography>
              <Typography variant="body1" color="text.secondary">
                View and manage your payment transactions
              </Typography>
            </Box>
          </Stack>
        </Box>

        {/* Filters and search */}
        <Card>
          <CardContent>
            <Stack direction="row" spacing={2} alignItems="center" flexWrap="wrap">
              <TextField
                placeholder="Search payments..."
                value={searchTerm}
                onChange={(e) => setSearchTerm(e.target.value)}
                size="small"
                InputProps={{
                  startAdornment: (
                    <InputAdornment position="start">
                      <Search />
                    </InputAdornment>
                  ),
                }}
                sx={{ minWidth: 250 }}
              />

              <LocalizationProvider dateAdapter={AdapterDateFns}>
                <DatePicker
                  label="From Date"
                  value={filters.dateFrom}
                  onChange={(date) => setFilters(prev => ({ ...prev, dateFrom: date }))}
                  slotProps={{ textField: { size: 'small' } }}
                />
                <DatePicker
                  label="To Date"
                  value={filters.dateTo}
                  onChange={(date) => setFilters(prev => ({ ...prev, dateTo: date }))}
                  slotProps={{ textField: { size: 'small' } }}
                />
              </LocalizationProvider>

              <IconButton
                onClick={(e) => setFilterAnchor(e.currentTarget)}
                color="primary"
              >
                <FilterList />
              </IconButton>

              <Button
                variant="outlined"
                startIcon={<Download />}
                onClick={handleExportData}
                disabled={filteredPayments.length === 0}
              >
                Export
              </Button>
            </Stack>
          </CardContent>
        </Card>

        {/* Payments table */}
        <Card>
          <TableContainer>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Date</TableCell>
                  <TableCell>Description</TableCell>
                  <TableCell>Type</TableCell>
                  <TableCell>Amount</TableCell>
                  <TableCell>Status</TableCell>
                  <TableCell>Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {filteredPayments
                  .slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage)
                  .map((payment) => (
                    <TableRow key={payment.id} hover>
                      <TableCell>
                        {formatDate(payment.createdAt)}
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" fontWeight={500}>
                          {payment.description}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          ID: {payment.id}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={getTypeLabel(payment.type)}
                          size="small"
                          variant="outlined"
                        />
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2" fontWeight={600}>
                          ${payment.amount.toFixed(2)} {payment.currency}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Chip
                          icon={getStatusIcon(payment.status)}
                          label={payment.status.charAt(0).toUpperCase() + payment.status.slice(1)}
                          color={getStatusColor(payment.status) as any}
                          size="small"
                        />
                      </TableCell>
                      <TableCell>
                        <IconButton
                          size="small"
                          onClick={() => handleViewDetails(payment)}
                        >
                          <Visibility />
                        </IconButton>
                        <IconButton size="small">
                          <Receipt />
                        </IconButton>
                      </TableCell>
                    </TableRow>
                  ))}
              </TableBody>
            </Table>
          </TableContainer>
          
          <TablePagination
            component="div"
            count={filteredPayments.length}
            page={page}
            onPageChange={(_, newPage) => setPage(newPage)}
            rowsPerPage={rowsPerPage}
            onRowsPerPageChange={(e) => {
              setRowsPerPage(parseInt(e.target.value, 10));
              setPage(0);
            }}
          />
        </Card>

        {filteredPayments.length === 0 && (
          <Card>
            <CardContent>
              <Box sx={{ textAlign: 'center', py: 4 }}>
                <History sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                <Typography variant="h6" gutterBottom>
                  No payments found
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {searchTerm || filters.status !== 'all' || filters.type !== 'all'
                    ? 'Try adjusting your search or filters'
                    : "You haven't made any payments yet"
                  }
                </Typography>
              </Box>
            </CardContent>
          </Card>
        )}
      </Stack>

      {/* Filter menu */}
      <Menu
        anchorEl={filterAnchor}
        open={Boolean(filterAnchor)}
        onClose={() => setFilterAnchor(null)}
      >
        <MenuItem onClick={() => {
          setFilters(prev => ({ ...prev, status: 'all' }));
          setFilterAnchor(null);
        }}>
          All Statuses
        </MenuItem>
        <MenuItem onClick={() => {
          setFilters(prev => ({ ...prev, status: 'completed' }));
          setFilterAnchor(null);
        }}>
          Completed Only
        </MenuItem>
        <MenuItem onClick={() => {
          setFilters(prev => ({ ...prev, status: 'pending' }));
          setFilterAnchor(null);
        }}>
          Pending Only
        </MenuItem>
        <MenuItem onClick={() => {
          setFilters(prev => ({ ...prev, status: 'failed' }));
          setFilterAnchor(null);
        }}>
          Failed Only
        </MenuItem>
      </Menu>

      {/* Payment details dialog */}
      <Dialog
        open={showDetailsDialog}
        onClose={() => setShowDetailsDialog(false)}
        maxWidth="md"
        fullWidth
      >
        <DialogTitle>
          Payment Details
        </DialogTitle>
        
        <DialogContent>
          {selectedPayment && (
            <Stack spacing={3}>
              <Alert severity="info">
                <Typography variant="body2">
                  Payment ID: {selectedPayment.id}
                </Typography>
              </Alert>

              <Grid container spacing={2}>
                <Grid item xs={12} sm={6}>
                  <Typography variant="body2" color="text.secondary">
                    Date
                  </Typography>
                  <Typography variant="body1" fontWeight={500}>
                    {formatDate(selectedPayment.createdAt)}
                  </Typography>
                </Grid>
                
                <Grid item xs={12} sm={6}>
                  <Typography variant="body2" color="text.secondary">
                    Amount
                  </Typography>
                  <Typography variant="body1" fontWeight={500}>
                    ${selectedPayment.amount.toFixed(2)} {selectedPayment.currency}
                  </Typography>
                </Grid>
                
                <Grid item xs={12} sm={6}>
                  <Typography variant="body2" color="text.secondary">
                    Type
                  </Typography>
                  <Typography variant="body1" fontWeight={500}>
                    {getTypeLabel(selectedPayment.type)}
                  </Typography>
                </Grid>
                
                <Grid item xs={12} sm={6}>
                  <Typography variant="body2" color="text.secondary">
                    Status
                  </Typography>
                  <Chip
                    icon={getStatusIcon(selectedPayment.status)}
                    label={selectedPayment.status.charAt(0).toUpperCase() + selectedPayment.status.slice(1)}
                    color={getStatusColor(selectedPayment.status) as any}
                    size="small"
                  />
                </Grid>
                
                <Grid item xs={12}>
                  <Typography variant="body2" color="text.secondary">
                    Description
                  </Typography>
                  <Typography variant="body1" fontWeight={500}>
                    {selectedPayment.description}
                  </Typography>
                </Grid>

                {selectedPayment.stripePaymentIntentId && (
                  <Grid item xs={12}>
                    <Typography variant="body2" color="text.secondary">
                      Stripe Payment Intent ID
                    </Typography>
                    <Typography variant="body1" fontFamily="monospace">
                      {selectedPayment.stripePaymentIntentId}
                    </Typography>
                  </Grid>
                )}
              </Grid>
            </Stack>
          )}
        </DialogContent>
        
        <DialogActions>
          <Button onClick={() => setShowDetailsDialog(false)}>
            Close
          </Button>
          <Button variant="outlined" startIcon={<Receipt />}>
            Download Receipt
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default PaymentHistory;