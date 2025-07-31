import React, { useState } from 'react';
import {
  Container,
  Grid,
  Paper,
  Typography,
  Box,
  Stack,
  Tab,
  Tabs,
  Divider,
} from '@mui/material';
import {
  Person,
  Security,
  Notifications,
  Payment,
} from '@mui/icons-material';

import ProfileEditor from '../../components/Profile/ProfileEditor';
import PasswordChange from '../../components/Profile/PasswordChange';
import NotificationSettings from '../../components/Profile/NotificationSettings';
import AccountSecurity from '../../components/Profile/AccountSecurity';

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
      id={`profile-tabpanel-${index}`}
      aria-labelledby={`profile-tab-${index}`}
    >
      {value === index && <Box>{children}</Box>}
    </div>
  );
};

const ProfilePage: React.FC = () => {
  const [activeTab, setActiveTab] = useState(0);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setActiveTab(newValue);
  };

  const tabs = [
    {
      label: 'Profile',
      icon: <Person />,
      component: <ProfileEditor />,
    },
    {
      label: 'Security',
      icon: <Security />,
      component: (
        <Stack spacing={4}>
          <PasswordChange />
          <AccountSecurity />
        </Stack>
      ),
    },
    {
      label: 'Notifications',
      icon: <Notifications />,
      component: <NotificationSettings />,
    },
  ];

  return (
    <Container maxWidth="lg" sx={{ py: 4 }}>
      {/* Header */}
      <Box sx={{ mb: 4 }}>
        <Typography variant="h4" component="h1" gutterBottom fontWeight={600}>
          Account Settings
        </Typography>
        <Typography variant="body1" color="text.secondary">
          Manage your profile, security, and notification preferences
        </Typography>
      </Box>

      <Grid container spacing={4}>
        {/* Sidebar with tabs */}
        <Grid item xs={12} md={3}>
          <Paper sx={{ p: 2 }}>
            <Tabs
              orientation="vertical"
              variant="scrollable"
              value={activeTab}
              onChange={handleTabChange}
              sx={{
                borderRight: 1,
                borderColor: 'divider',
                '& .MuiTab-root': {
                  alignItems: 'flex-start',
                  textAlign: 'left',
                  minHeight: 48,
                },
              }}
            >
              {tabs.map((tab, index) => (
                <Tab
                  key={index}
                  icon={tab.icon}
                  iconPosition="start"
                  label={tab.label}
                  id={`profile-tab-${index}`}
                  aria-controls={`profile-tabpanel-${index}`}
                  sx={{
                    justifyContent: 'flex-start',
                    gap: 2,
                  }}
                />
              ))}
            </Tabs>
          </Paper>
        </Grid>

        {/* Main content */}
        <Grid item xs={12} md={9}>
          <Paper sx={{ p: 4 }}>
            {tabs.map((tab, index) => (
              <TabPanel key={index} value={activeTab} index={index}>
                <Box sx={{ mb: 3 }}>
                  <Stack direction="row" alignItems="center" spacing={2} mb={1}>
                    {tab.icon}
                    <Typography variant="h5" fontWeight={600}>
                      {tab.label}
                    </Typography>
                  </Stack>
                  <Divider />
                </Box>
                {tab.component}
              </TabPanel>
            ))}
          </Paper>
        </Grid>
      </Grid>
    </Container>
  );
};

export default ProfilePage;