import React, { useState } from 'react';
import {
  Box,
  Stack,
  Typography,
  Card,
  CardContent,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TablePagination,
  Chip,
  IconButton,
  TextField,
  InputAdornment,
  Menu,
  MenuItem,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Alert,
} from '@mui/material';
import {
  History,
  Search,
  FilterList,
  Visibility,
  Download,
  TrendingUp,
  TrendingDown,
  Add,
  Remove,
  Schedule,
  Refresh,
} from '@mui/icons-material';
import { DatePicker } from '@mui/x-date-pickers/DatePicker';
import { LocalizationProvider } from '@mui/x-date-pickers/LocalizationProvider';
import { AdapterDateFns } from '@mui/x-date-pickers/AdapterDateFns';

interface CreditTransaction {
  id: string;
  type: 'earned' | 'spent' | 'expired' | 'refunded' | 'purchased' | 'redeemed';
  amount: number;
  description: string;
  date: string;
  raffleId?: string;
  paymentId?: string;
  status: 'completed' | 'pending' | 'failed';
  balanceAfter: number;
}

const CreditTransactions: React.FC = () => {
  const [page, setPage] = useState(0);
  const [rowsPerPage, setRowsPerPage] = useState(10);
  const [searchTerm, setSearchTerm] = useState('');
  const [filterAnchor, setFilterAnchor] = useState<null | HTMLElement>(null);
  const [selectedTransaction, setSelectedTransaction] = useState<CreditTransaction | null>(null);
  const [showDetailsDialog, setShowDetailsDialog] = useState(false);
  const [filters, setFilters] = useState({
    type: 'all' as string,
    dateFrom: null as Date | null,
    dateTo: null as Date | null,
  });

  // Mock transaction data - in real app, this would come from API
  const transactions: CreditTransaction[] = [
    {
      id: 'ct_1',
      type: 'purchased',
      amount: 115,
      description: 'Credit purchase via Stripe - $100 + $15 bonus',
      date: '2024-01-18T10:30:00Z',
      paymentId: 'pay_123',
      status: 'completed',
      balanceAfter: 215,
    },
    {
      id: 'ct_2',
      type: 'spent',
      amount: -60,
      description: 'iPhone 15 Pro Max raffle - 5 boxes purchased',
      date: '2024-01-17T15:45:00Z',
      raffleId: 'raffle_1',
      status: 'completed',
      balanceAfter: 100,
    },
    {
      id: 'ct_3',
      type: 'earned',
      amount: 45,
      description: 'MacBook Pro raffle - non-winner credit return',
      date: '2024-01-16T09:20:00Z',
      raffleId: 'raffle_2',
      status: 'completed',
      balanceAfter: 160,
    },
    {
      id: 'ct_4',
      type: 'redeemed',
      amount: -25,
      description: 'Redeemed for Wireless Bluetooth Earbuds',
      date: '2024-01-15T14:10:00Z',
      status: 'completed',
      balanceAfter: 115,
    },
    {
      id: 'ct_5',
      type: 'purchased',
      amount: 55,
      description: 'Credit purchase via PayPal - $50 + $5 bonus',
      date: '2024-01-14T11:00:00Z',
      paymentId: 'pay_456',
      status: 'completed',
      balanceAfter: 140,
    },
    {
      id: 'ct_6',
      type: 'expired',
      amount: -15,
      description: 'Item-specific credits expired - Nintendo Switch',
      date: '2024-01-13T00:00:00Z',
      status: 'completed',
      balanceAfter: 85,
    },
    {
      id: 'ct_7',
      type: 'refunded',
      amount: 20,
      description: 'Refund for cancelled Samsung Galaxy Watch raffle',
      date: '2024-01-12T16:30:00Z',
      raffleId: 'raffle_3',
      status: 'completed',
      balanceAfter: 100,
    },
    {
      id: 'ct_8',
      type: 'spent',
      amount: -35,
      description: 'Sony WH-1000XM5 raffle - 3 boxes purchased',
      date: '2024-01-11T13:20:00Z',
      raffleId: 'raffle_4',
      status: 'completed',
      balanceAfter: 80,
    },
  ];

  // Filter transactions
  const filteredTransactions = transactions.filter(transaction => {
    const matchesSearch = transaction.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         transaction.id.toLowerCase().includes(searchTerm.toLowerCase());
    
    const matchesType = filters.type === 'all' || transaction.type === filters.type;
    
    const transactionDate = new Date(transaction.date);
    const matchesDateFrom = !filters.dateFrom || transactionDate >= filters.dateFrom;
    const matchesDateTo = !filters.dateTo || transactionDate <= filters.dateTo;
    
    return matchesSearch && matchesType && matchesDateFrom && matchesDateTo;
  });

  const getTransactionIcon = (type: string) => {
    switch (type) {
      case 'earned': return <TrendingUp color="success" />;
      case 'spent': return <TrendingDown color="error" />;
      case 'purchased': return <Add color="primary" />;
      case 'redeemed': return <Remove color="warning" />;
      case 'expired': return <Schedule color="error" />;
      case 'refunded': return <Refresh color="info" />;
      default: return <History />;
    }
  };

  const getTransactionColor = (type: string, amount: number) => {
    if (amount > 0) return 'success.main';
    if (amount < 0) return 'error.main';
    return 'text.primary';
  };

  const getTypeLabel = (type: string) => {
    switch (type) {
      case 'earned': return 'Earned';
      case 'spent': return 'Spent';
      case 'purchased': return 'Purchased';
      case 'redeemed': return 'Redeemed';
      case 'expired': return 'Expired';
      case 'refunded': return 'Refunded';
      default: return type;
    }
  };

  const formatAmount = (amount: number) => {
    const sign = amount >= 0 ? '+' : '';
    return `${sign}$${Math.abs(amount).toFixed(2)}`;
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

  const handleViewDetails = (transaction: CreditTransaction) => {
    setSelectedTransaction(transaction);
    setShowDetailsDialog(true);
  };

  const handleExportData = () => {
    // Create CSV data
    const csvData = filteredTransactions.map(transaction => ({
      ID: transaction.id,
      Date: formatDate(transaction.date),
      Type: getTypeLabel(transaction.type),
      Description: transaction.description,
      Amount: formatAmount(transaction.amount),
      'Balance After': `$${transaction.balanceAfter.toFixed(2)}`,
      Status: transaction.status,
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
    a.download = `credit-transactions-${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
    window.URL.revokeObjectURL(url);
  };

  // Calculate summary statistics
  const totalEarned = transactions
    .filter(t => t.amount > 0)
    .reduce((sum, t) => sum + t.amount, 0);
  
  const totalSpent = Math.abs(transactions
    .filter(t => t.amount < 0)
    .reduce((sum, t) => sum + t.amount, 0));

  return (
    <Box>
      <Stack spacing={4}>
        {/* Header */}
        <Box>
          <Stack direction="row" alignItems="center" spacing={2} mb={2}>
            <History color="primary" sx={{ fontSize: 32 }} />
            <Box>
              <Typography variant="h5" fontWeight={600}>
                Credit Transactions
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Complete history of all your credit activities
              </Typography>
            </Box>
          </Stack>
        </Box>

        {/* Summary cards */}
        <Stack direction="row" spacing={3}>
          <Card sx={{ flex: 1 }}>
            <CardContent>
              <Stack alignItems="center" spacing={1}>
                <TrendingUp sx={{ fontSize: 32, color: 'success.main' }} />
                <Typography variant="h5" fontWeight={600} color="success.main">
                  ${totalEarned.toFixed(2)}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Total Earned
                </Typography>
              </Stack>
            </CardContent>
          </Card>

          <Card sx={{ flex: 1 }}>
            <CardContent>
              <Stack alignItems="center" spacing={1}>
                <TrendingDown sx={{ fontSize: 32, color: 'error.main' }} />
                <Typography variant="h5" fontWeight={600} color="error.main">
                  ${totalSpent.toFixed(2)}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Total Spent
                </Typography>
              </Stack>
            </CardContent>
          </Card>

          <Card sx={{ flex: 1 }}>
            <CardContent>
              <Stack alignItems="center" spacing={1}>
                <History sx={{ fontSize: 32, color: 'primary.main' }} />
                <Typography variant="h5" fontWeight={600} color="primary.main">
                  {transactions.length}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Total Transactions
                </Typography>
              </Stack>
            </CardContent>
          </Card>
        </Stack>

        {/* Filters and search */}
        <Card>
          <CardContent>
            <Stack direction="row" spacing={2} alignItems="center" flexWrap="wrap">
              <TextField
                placeholder="Search transactions..."
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
                disabled={filteredTransactions.length === 0}
              >
                Export
              </Button>
            </Stack>
          </CardContent>
        </Card>

        {/* Transactions table */}
        <Card>
          <TableContainer>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Date</TableCell>
                  <TableCell>Type</TableCell>
                  <TableCell>Description</TableCell>
                  <TableCell align="right">Amount</TableCell>
                  <TableCell align="right">Balance After</TableCell>
                  <TableCell>Status</TableCell>
                  <TableCell>Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {filteredTransactions
                  .slice(page * rowsPerPage, page * rowsPerPage + rowsPerPage)
                  .map((transaction) => (
                    <TableRow key={transaction.id} hover>
                      <TableCell>
                        {formatDate(transaction.date)}
                      </TableCell>
                      <TableCell>
                        <Stack direction="row" alignItems="center" spacing={1}>
                          {getTransactionIcon(transaction.type)}
                          <Chip
                            label={getTypeLabel(transaction.type)}
                            size="small"
                            variant="outlined"
                          />
                        </Stack>
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2">
                          {transaction.description}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          ID: {transaction.id}
                        </Typography>
                      </TableCell>
                      <TableCell align="right">
                        <Typography
                          variant="body2"
                          fontWeight={600}
                          color={getTransactionColor(transaction.type, transaction.amount)}
                        >
                          {formatAmount(transaction.amount)}
                        </Typography>
                      </TableCell>
                      <TableCell align="right">
                        <Typography variant="body2" fontWeight={500}>
                          ${transaction.balanceAfter.toFixed(2)}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={transaction.status.charAt(0).toUpperCase() + transaction.status.slice(1)}
                          color={transaction.status === 'completed' ? 'success' : 'warning'}
                          size="small"
                        />
                      </TableCell>
                      <TableCell>
                        <IconButton
                          size="small"
                          onClick={() => handleViewDetails(transaction)}
                        >
                          <Visibility />
                        </IconButton>
                      </TableCell>
                    </TableRow>
                  ))}
              </TableBody>
            </Table>
          </TableContainer>
          
          <TablePagination
            component="div"
            count={filteredTransactions.length}
            page={page}
            onPageChange={(_, newPage) => setPage(newPage)}
            rowsPerPage={rowsPerPage}
            onRowsPerPageChange={(e) => {
              setRowsPerPage(parseInt(e.target.value, 10));
              setPage(0);
            }}
          />
        </Card>

        {filteredTransactions.length === 0 && (
          <Card>
            <CardContent>
              <Box sx={{ textAlign: 'center', py: 4 }}>
                <History sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
                <Typography variant="h6" gutterBottom>
                  No transactions found
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  {searchTerm || filters.type !== 'all'
                    ? 'Try adjusting your search or filters'
                    : "You haven't made any credit transactions yet"
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
          setFilters(prev => ({ ...prev, type: 'all' }));
          setFilterAnchor(null);
        }}>
          All Types
        </MenuItem>
        <MenuItem onClick={() => {
          setFilters(prev => ({ ...prev, type: 'earned' }));
          setFilterAnchor(null);
        }}>
          Earned Only
        </MenuItem>
        <MenuItem onClick={() => {
          setFilters(prev => ({ ...prev, type: 'spent' }));
          setFilterAnchor(null);
        }}>
          Spent Only
        </MenuItem>
        <MenuItem onClick={() => {
          setFilters(prev => ({ ...prev, type: 'purchased' }));
          setFilterAnchor(null);
        }}>
          Purchased Only
        </MenuItem>
      </Menu>

      {/* Transaction details dialog */}
      <Dialog
        open={showDetailsDialog}
        onClose={() => setShowDetailsDialog(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          Transaction Details
        </DialogTitle>
        
        <DialogContent>
          {selectedTransaction && (
            <Stack spacing={3}>
              <Alert severity="info">
                <Typography variant="body2">
                  Transaction ID: {selectedTransaction.id}
                </Typography>
              </Alert>

              <Stack spacing={2}>
                <Stack direction="row" justifyContent="space-between">
                  <Typography color="text.secondary">Date</Typography>
                  <Typography fontWeight={500}>
                    {formatDate(selectedTransaction.date)}
                  </Typography>
                </Stack>
                
                <Stack direction="row" justifyContent="space-between">
                  <Typography color="text.secondary">Type</Typography>
                  <Chip
                    label={getTypeLabel(selectedTransaction.type)}
                    size="small"
                    variant="outlined"
                  />
                </Stack>
                
                <Stack direction="row" justifyContent="space-between">
                  <Typography color="text.secondary">Amount</Typography>
                  <Typography
                    fontWeight={600}
                    color={getTransactionColor(selectedTransaction.type, selectedTransaction.amount)}
                  >
                    {formatAmount(selectedTransaction.amount)}
                  </Typography>
                </Stack>
                
                <Stack direction="row" justifyContent="space-between">
                  <Typography color="text.secondary">Balance After</Typography>
                  <Typography fontWeight={500}>
                    ${selectedTransaction.balanceAfter.toFixed(2)}
                  </Typography>
                </Stack>
                
                <Stack direction="row" justifyContent="space-between">
                  <Typography color="text.secondary">Status</Typography>
                  <Chip
                    label={selectedTransaction.status.charAt(0).toUpperCase() + selectedTransaction.status.slice(1)}
                    color={selectedTransaction.status === 'completed' ? 'success' : 'warning'}
                    size="small"
                  />
                </Stack>
                
                <Box>
                  <Typography color="text.secondary" gutterBottom>
                    Description
                  </Typography>
                  <Typography variant="body1">
                    {selectedTransaction.description}
                  </Typography>
                </Box>

                {selectedTransaction.raffleId && (
                  <Stack direction="row" justifyContent="space-between">
                    <Typography color="text.secondary">Raffle ID</Typography>
                    <Typography fontFamily="monospace">
                      {selectedTransaction.raffleId}
                    </Typography>
                  </Stack>
                )}

                {selectedTransaction.paymentId && (
                  <Stack direction="row" justifyContent="space-between">
                    <Typography color="text.secondary">Payment ID</Typography>
                    <Typography fontFamily="monospace">
                      {selectedTransaction.paymentId}
                    </Typography>
                  </Stack>
                )}
              </Stack>
            </Stack>
          )}
        </DialogContent>
        
        <DialogActions>
          <Button onClick={() => setShowDetailsDialog(false)}>
            Close
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default CreditTransactions;