import React from 'react';
import { Box, Typography } from '@mui/material';

const WalletPage: React.FC = () => {
  return (
    <Box
      sx={{
        display: 'flex',
        justifyContent: 'center',
        alignItems: 'center',
        minHeight: '60vh',
      }}
    >
      <Typography variant="h4" color="text.secondary">
        Wallet Page - Coming Soon
      </Typography>
    </Box>
  );
};

export default WalletPage;