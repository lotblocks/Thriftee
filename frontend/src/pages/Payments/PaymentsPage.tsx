import React, { useState } from 'react';
import {
  Container,
  Box,
  Typography,
  Tabs,
  Tab,
  Paper,
  Stack,
  Divider,
} from '@mui/material';
import {
  CreditCard,
  History,
  AccountBalanceWallet,
} from '@mui/icons-material';

import CreditPurchase from '../../components/Payment/CreditPurchase';
import PaymentHistory from '../../components/Payment/PaymentHistory';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

const TabPanel: React.FC<TabPanelProps> = ({ children, value, index }) => {
  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`payments-tabpanel-${index}`}
      aria-labelledby={`payments-tab-${index}`}
    >
      {value === index && <Box>{children}</Box>}
    </div>
  );
};

const PaymentsPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const tabs = [
    {
      label: 'Buy Credits',
      icon: <AccountBalanceWallet />,
      component: <CreditPurchase />,
    },
    {
      label: 'Payment History',
      icon: <History />,
      component: <PaymentHistory />,
    },
  ];

  return (
    <Container maxWidth="xl" sx={{ py: 4 }}>
      {/* Header */}
      <Box sx={{ mb: 4 }}>
        <Stack direction="row" alignItems="center" spacing={2} mb={2}>
          <CreditCard color="primary" sx={{ fontSize: 32 }} />
          <Box>
            <Typography variant="h4" component="h1" fontWeight={600}>
              Payments & Credits
            </Typography>
            <Typography variant="body1" color="text.secondary">
              Manage your credits and view payment history
            </Typography>
          </Box>
        </Stack>
      </Box>

      {/* Tabs */}
      <Paper sx={{ mb: 3 }}>
        <Tabs
          value={activeTab}
          onChange={handleTabChange}
          sx={{
            borderBottom: 1,
            borderColor: 'divider',
            '& .MuiTab-root': {
              minHeight: 64,
              textTransform: 'none',
              fontSize: '1rem',
              fontWeight: 500,
            },
          }}
        >
          {tabs.map((tab, index) => (
            <Tab
              key={index}
              icon={tab.icon}
              iconPosition="start"
              label={tab.label}
              id={`payments-tab-${index}`}
              aria-controls={`payments-tabpanel-${index}`}
              sx={{
                gap: 1,
              }}
            />
          ))}
        </Tabs>
      </Paper>

      {/* Tab content */}
      {tabs.map((tab, index) => (
        <TabPanel key={index} value={activeTab} index={index}>
          {tab.component}
        </TabPanel>
      ))}
    </Container>
  );
};

export default PaymentsPage;