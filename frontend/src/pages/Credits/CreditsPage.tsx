import React, { useState } from 'react';
import {
  Container,
  Box,
  Typography,
  Tabs,
  Tab,
  Paper,
  Stack,
} from '@mui/material';
import {
  AccountBalanceWallet,
  LocalOffer,
  History,
  Schedule,
} from '@mui/icons-material';

import CreditBalance from '../../components/Dashboard/CreditBalance';
import CreditRedemption from '../../components/Credits/CreditRedemption';
import ExpiringCredits from '../../components/Credits/ExpiringCredits';
import CreditTransactions from '../../components/Credits/CreditTransactions';

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
      id={`credits-tabpanel-${index}`}
      aria-labelledby={`credits-tab-${index}`}
    >
      {value === index && <Box>{children}</Box>}
    </div>
  );
};

const CreditsPage: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const tabs = [
    {
      label: 'Balance Overview',
      icon: <AccountBalanceWallet />,
      component: <CreditBalance />,
    },
    {
      label: 'Redeem Credits',
      icon: <LocalOffer />,
      component: <CreditRedemption />,
    },
    {
      label: 'Expiring Soon',
      icon: <Schedule />,
      component: <ExpiringCredits />,
    },
    {
      label: 'Transaction History',
      icon: <History />,
      component: <CreditTransactions />,
    },
  ];

  return (
    <Container maxWidth="xl" sx={{ py: 4 }}>
      {/* Header */}
      <Box sx={{ mb: 4 }}>
        <Stack direction="row" alignItems="center" spacing={2} mb={2}>
          <AccountBalanceWallet color="primary" sx={{ fontSize: 32 }} />
          <Box>
            <Typography variant="h4" component="h1" fontWeight={600}>
              Credit Management
            </Typography>
            <Typography variant="body1" color="text.secondary">
              Manage your credits, redeem rewards, and track transactions
            </Typography>
          </Box>
        </Stack>
      </Box>

      {/* Tabs */}
      <Paper sx={{ mb: 3 }}>
        <Tabs
          value={activeTab}
          onChange={handleTabChange}
          variant="scrollable"
          scrollButtons="auto"
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
              id={`credits-tab-${index}`}
              aria-controls={`credits-tabpanel-${index}`}
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

export default CreditsPage;