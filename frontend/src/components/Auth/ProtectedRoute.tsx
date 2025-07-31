import React, { useEffect } from 'react';
import { Navigate, useLocation } from 'react-router-dom';
import { Box } from '@mui/material';

import { useAppSelector, useAppDispatch } from '../../store';
import { getCurrentUser } from '../../store/slices/authSlice';
import LoadingSpinner from '../UI/LoadingSpinner';

interface ProtectedRouteProps {
  children: React.ReactNode;
  requireAdmin?: boolean;
}

const ProtectedRoute: React.FC<ProtectedRouteProps> = ({ 
  children, 
  requireAdmin = false 
}) => {
  const dispatch = useAppDispatch();
  const location = useLocation();
  const { isAuthenticated, user, isLoading, token } = useAppSelector(state => state.auth);

  useEffect(() => {
    // If we have a token but no user, try to get current user
    if (token && !user && !isLoading) {
      dispatch(getCurrentUser());
    }
  }, [dispatch, token, user, isLoading]);

  // Show loading while checking authentication
  if (isLoading) {
    return (
      <Box
        sx={{
          display: 'flex',
          justifyContent: 'center',
          alignItems: 'center',
          minHeight: '60vh',
        }}
      >
        <LoadingSpinner message="Checking authentication..." />
      </Box>
    );
  }

  // Redirect to login if not authenticated
  if (!isAuthenticated || !user) {
    return <Navigate to="/login" state={{ from: location }} replace />;
  }

  // Check admin requirement
  if (requireAdmin && user.role !== 'admin') {
    return <Navigate to="/dashboard" replace />;
  }

  return <>{children}</>;
};

export default ProtectedRoute;