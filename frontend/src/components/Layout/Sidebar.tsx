import React from 'react';
import {
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Divider,
  Box,
  Typography,
} from '@mui/material';
import {
  Dashboard,
  Casino,
  AccountBalanceWallet,
  CreditCard,
  Person,
  AdminPanelSettings,
  Settings,
  Help,
} from '@mui/icons-material';
import { useNavigate, useLocation } from 'react-router-dom';

import { useAppSelector, useAppDispatch } from '../../store';
import { setSidebarOpen } from '../../store/slices/uiSlice';

interface NavigationItem {
  text: string;
  icon: React.ReactElement;
  path: string;
  adminOnly?: boolean;
}

const navigationItems: NavigationItem[] = [
  {
    text: 'Dashboard',
    icon: <Dashboard />,
    path: '/dashboard',
  },
  {
    text: 'Raffles',
    icon: <Casino />,
    path: '/raffles',
  },
  {
    text: 'Credits',
    icon: <AccountBalanceWallet />,
    path: '/credits',
  },
  {
    text: 'Payments',
    icon: <CreditCard />,
    path: '/payments',
  },
  {
    text: 'Wallet',
    icon: <AccountBalanceWallet />,
    path: '/wallet',
  },
  {
    text: 'Profile',
    icon: <Person />,
    path: '/profile',
  },
];

const adminItems: NavigationItem[] = [
  {
    text: 'Admin Panel',
    icon: <AdminPanelSettings />,
    path: '/admin',
    adminOnly: true,
  },
];

const Sidebar: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const dispatch = useAppDispatch();
  const { sidebarOpen } = useAppSelector(state => state.ui);
  const { user } = useAppSelector(state => state.auth);

  const handleItemClick = (path: string) => {
    navigate(path);
    // Close sidebar on mobile after navigation
    if (window.innerWidth < 900) {
      dispatch(setSidebarOpen(false));
    }
  };

  const isAdmin = user?.role === 'admin';
  const drawerWidth = 240;

  const drawer = (
    <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
      <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
        <Typography variant="h6" color="primary" fontWeight={600}>
          Navigation
        </Typography>
      </Box>

      <List sx={{ flex: 1, py: 1 }}>
        {navigationItems.map((item) => {
          const isActive = location.pathname === item.path;
          return (
            <ListItem key={item.text} disablePadding>
              <ListItemButton
                onClick={() => handleItemClick(item.path)}
                selected={isActive}
                sx={{
                  mx: 1,
                  borderRadius: 1,
                  '&.Mui-selected': {
                    backgroundColor: 'primary.main',
                    color: 'white',
                    '&:hover': {
                      backgroundColor: 'primary.dark',
                    },
                    '& .MuiListItemIcon-root': {
                      color: 'white',
                    },
                  },
                }}
              >
                <ListItemIcon
                  sx={{
                    color: isActive ? 'white' : 'text.secondary',
                    minWidth: 40,
                  }}
                >
                  {item.icon}
                </ListItemIcon>
                <ListItemText
                  primary={item.text}
                  primaryTypographyProps={{
                    fontSize: '0.875rem',
                    fontWeight: isActive ? 600 : 400,
                  }}
                />
              </ListItemButton>
            </ListItem>
          );
        })}

        {isAdmin && (
          <>
            <Divider sx={{ my: 1, mx: 2 }} />
            <Box sx={{ px: 2, py: 1 }}>
              <Typography variant="caption" color="text.secondary" fontWeight={600}>
                ADMIN
              </Typography>
            </Box>
            {adminItems.map((item) => {
              const isActive = location.pathname.startsWith(item.path);
              return (
                <ListItem key={item.text} disablePadding>
                  <ListItemButton
                    onClick={() => handleItemClick(item.path)}
                    selected={isActive}
                    sx={{
                      mx: 1,
                      borderRadius: 1,
                      '&.Mui-selected': {
                        backgroundColor: 'secondary.main',
                        color: 'white',
                        '&:hover': {
                          backgroundColor: 'secondary.dark',
                        },
                        '& .MuiListItemIcon-root': {
                          color: 'white',
                        },
                      },
                    }}
                  >
                    <ListItemIcon
                      sx={{
                        color: isActive ? 'white' : 'text.secondary',
                        minWidth: 40,
                      }}
                    >
                      {item.icon}
                    </ListItemIcon>
                    <ListItemText
                      primary={item.text}
                      primaryTypographyProps={{
                        fontSize: '0.875rem',
                        fontWeight: isActive ? 600 : 400,
                      }}
                    />
                  </ListItemButton>
                </ListItem>
              );
            })}
          </>
        )}
      </List>

      <Divider />
      <List>
        <ListItem disablePadding>
          <ListItemButton sx={{ mx: 1, borderRadius: 1 }}>
            <ListItemIcon sx={{ minWidth: 40 }}>
              <Settings />
            </ListItemIcon>
            <ListItemText
              primary="Settings"
              primaryTypographyProps={{ fontSize: '0.875rem' }}
            />
          </ListItemButton>
        </ListItem>
        <ListItem disablePadding>
          <ListItemButton sx={{ mx: 1, borderRadius: 1 }}>
            <ListItemIcon sx={{ minWidth: 40 }}>
              <Help />
            </ListItemIcon>
            <ListItemText
              primary="Help"
              primaryTypographyProps={{ fontSize: '0.875rem' }}
            />
          </ListItemButton>
        </ListItem>
      </List>
    </Box>
  );

  return (
    <Drawer
      variant="persistent"
      anchor="left"
      open={sidebarOpen}
      sx={{
        width: drawerWidth,
        flexShrink: 0,
        '& .MuiDrawer-paper': {
          width: drawerWidth,
          boxSizing: 'border-box',
          borderRight: 1,
          borderColor: 'divider',
          top: '64px', // Height of AppBar
          height: 'calc(100vh - 64px)',
        },
      }}
    >
      {drawer}
    </Drawer>
  );
};

export default Sidebar;