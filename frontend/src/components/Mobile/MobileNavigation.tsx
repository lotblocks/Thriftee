import React, { useState } from 'react';
import {
  BottomNavigation,
  BottomNavigationAction,
  Paper,
  Badge,
  Drawer,
  List,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Box,
  Typography,
  IconButton,
  Divider,
  Avatar,
  Stack,
} from '@mui/material';
import {
  Home,
  Casino,
  AccountBalanceWallet,
  Person,
  Menu,
  Close,
  Dashboard,
  CreditCard,
  Notifications,
  Settings,
  ExitToApp,
} from '@mui/icons-material';
import { useNavigate, useLocation } from 'react-router-dom';

import { useAppSelector, useAppDispatch } from '../../store';
import { logout } from '../../store/slices/authSlice';

const MobileNavigation: React.FC = () => {
  const navigate = useNavigate();
  const location = useLocation();
  const dispatch = useAppDispatch();
  const { isAuthenticated, user } = useAppSelector(state => state.auth);
  const { balance } = useAppSelector(state => state.credit);
  const [drawerOpen, setDrawerOpen] = useState(false);

  // Get current tab based on pathname
  const getCurrentTab = () => {
    const path = location.pathname;
    if (path === '/') return 0;
    if (path.startsWith('/raffles')) return 1;
    if (path.startsWith('/credits') || path.startsWith('/payments')) return 2;
    if (path.startsWith('/profile') || path.startsWith('/dashboard')) return 3;
    return 0;
  };

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    switch (newValue) {
      case 0:
        navigate('/');
        break;
      case 1:
        navigate('/raffles');
        break;
      case 2:
        navigate(isAuthenticated ? '/credits' : '/login');
        break;
      case 3:
        navigate(isAuthenticated ? '/dashboard' : '/login');
        break;
    }
  };

  const handleLogout = () => {
    dispatch(logout());
    setDrawerOpen(false);
    navigate('/');
  };

  const menuItems = [
    {
      text: 'Dashboard',
      icon: <Dashboard />,
      path: '/dashboard',
      requireAuth: true,
    },
    {
      text: 'My Raffles',
      icon: <Casino />,
      path: '/raffles',
      requireAuth: false,
    },
    {
      text: 'Credits',
      icon: <AccountBalanceWallet />,
      path: '/credits',
      requireAuth: true,
    },
    {
      text: 'Payments',
      icon: <CreditCard />,
      path: '/payments',
      requireAuth: true,
    },
    {
      text: 'Profile',
      icon: <Person />,
      path: '/profile',
      requireAuth: true,
    },
    {
      text: 'Settings',
      icon: <Settings />,
      path: '/settings',
      requireAuth: true,
    },
  ];

  return (
    <>
      {/* Bottom Navigation */}
      <Paper
        sx={{
          position: 'fixed',
          bottom: 0,
          left: 0,
          right: 0,
          zIndex: 1000,
          borderTop: 1,
          borderColor: 'divider',
        }}
        elevation={8}
      >
        <BottomNavigation
          value={getCurrentTab()}
          onChange={handleTabChange}
          showLabels
          sx={{
            height: 64,
            '& .MuiBottomNavigationAction-root': {
              minWidth: 'auto',
              padding: '6px 12px 8px',
            },
            '& .MuiBottomNavigationAction-label': {
              fontSize: '0.75rem',
              '&.Mui-selected': {
                fontSize: '0.75rem',
              },
            },
          }}
        >
          <BottomNavigationAction
            label="Home"
            icon={<Home />}
            sx={{
              color: getCurrentTab() === 0 ? 'primary.main' : 'text.secondary',
            }}
          />
          <BottomNavigationAction
            label="Raffles"
            icon={<Casino />}
            sx={{
              color: getCurrentTab() === 1 ? 'primary.main' : 'text.secondary',
            }}
          />
          <BottomNavigationAction
            label="Credits"
            icon={
              <Badge
                badgeContent={isAuthenticated ? `$${Math.floor(balance)}` : 0}
                color="primary"
                max={999}
                sx={{
                  '& .MuiBadge-badge': {
                    fontSize: '0.6rem',
                    height: 16,
                    minWidth: 16,
                  },
                }}
              >
                <AccountBalanceWallet />
              </Badge>
            }
            sx={{
              color: getCurrentTab() === 2 ? 'primary.main' : 'text.secondary',
            }}
          />
          <BottomNavigationAction
            label="Menu"
            icon={<Menu />}
            onClick={(e) => {
              e.stopPropagation();
              setDrawerOpen(true);
            }}
            sx={{
              color: 'text.secondary',
            }}
          />
        </BottomNavigation>
      </Paper>

      {/* Side Drawer Menu */}
      <Drawer
        anchor="right"
        open={drawerOpen}
        onClose={() => setDrawerOpen(false)}
        PaperProps={{
          sx: {
            width: 280,
            maxWidth: '80vw',
          },
        }}
      >
        <Box sx={{ height: '100%', display: 'flex', flexDirection: 'column' }}>
          {/* Header */}
          <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
            <Stack direction="row" alignItems="center" justifyContent="space-between">
              <Typography variant="h6" fontWeight={600}>
                Menu
              </Typography>
              <IconButton onClick={() => setDrawerOpen(false)}>
                <Close />
              </IconButton>
            </Stack>
          </Box>

          {/* User info */}
          {isAuthenticated && user && (
            <Box sx={{ p: 2, borderBottom: 1, borderColor: 'divider' }}>
              <Stack direction="row" alignItems="center" spacing={2}>
                <Avatar
                  src={user.profile?.avatar}
                  sx={{ width: 48, height: 48 }}
                >
                  {user.email?.charAt(0).toUpperCase()}
                </Avatar>
                <Box sx={{ flex: 1 }}>
                  <Typography variant="subtitle1" fontWeight={600}>
                    {user.profile?.firstName && user.profile?.lastName
                      ? `${user.profile.firstName} ${user.profile.lastName}`
                      : user.email?.split('@')[0]
                    }
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    ${balance.toFixed(2)} credits
                  </Typography>
                </Box>
              </Stack>
            </Box>
          )}

          {/* Menu items */}
          <List sx={{ flex: 1, py: 1 }}>
            {menuItems.map((item) => {
              if (item.requireAuth && !isAuthenticated) return null;
              
              const isActive = location.pathname === item.path;
              
              return (
                <ListItem key={item.text} disablePadding>
                  <ListItemButton
                    onClick={() => {
                      navigate(item.path);
                      setDrawerOpen(false);
                    }}
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

            {/* Notifications */}
            {isAuthenticated && (
              <ListItem disablePadding>
                <ListItemButton sx={{ mx: 1, borderRadius: 1 }}>
                  <ListItemIcon sx={{ minWidth: 40 }}>
                    <Badge badgeContent={3} color="error">
                      <Notifications />
                    </Badge>
                  </ListItemIcon>
                  <ListItemText
                    primary="Notifications"
                    primaryTypographyProps={{ fontSize: '0.875rem' }}
                  />
                </ListItemButton>
              </ListItem>
            )}
          </List>

          {/* Auth actions */}
          <Box sx={{ borderTop: 1, borderColor: 'divider' }}>
            {isAuthenticated ? (
              <ListItem disablePadding>
                <ListItemButton onClick={handleLogout} sx={{ mx: 1, borderRadius: 1 }}>
                  <ListItemIcon sx={{ minWidth: 40 }}>
                    <ExitToApp />
                  </ListItemIcon>
                  <ListItemText
                    primary="Logout"
                    primaryTypographyProps={{ fontSize: '0.875rem' }}
                  />
                </ListItemButton>
              </ListItem>
            ) : (
              <Stack spacing={1} sx={{ p: 2 }}>
                <Typography variant="body2" color="text.secondary" textAlign="center">
                  Sign in to access all features
                </Typography>
                <Stack direction="row" spacing={1}>
                  <Box sx={{ flex: 1 }}>
                    <BottomNavigationAction
                      label="Login"
                      onClick={() => {
                        navigate('/login');
                        setDrawerOpen(false);
                      }}
                      sx={{
                        width: '100%',
                        border: 1,
                        borderColor: 'primary.main',
                        borderRadius: 1,
                        color: 'primary.main',
                      }}
                    />
                  </Box>
                  <Box sx={{ flex: 1 }}>
                    <BottomNavigationAction
                      label="Sign Up"
                      onClick={() => {
                        navigate('/register');
                        setDrawerOpen(false);
                      }}
                      sx={{
                        width: '100%',
                        backgroundColor: 'primary.main',
                        borderRadius: 1,
                        color: 'white',
                        '&:hover': {
                          backgroundColor: 'primary.dark',
                        },
                      }}
                    />
                  </Box>
                </Stack>
              </Stack>
            )}
          </Box>
        </Box>
      </Drawer>
    </>
  );
};

export default MobileNavigation;