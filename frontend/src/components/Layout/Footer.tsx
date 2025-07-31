import React from 'react';
import {
  Box,
  Container,
  Typography,
  Link,
  Grid,
  IconButton,
  Divider,
} from '@mui/material';
import {
  Twitter,
  Facebook,
  Instagram,
  LinkedIn,
  GitHub,
} from '@mui/icons-material';

const Footer: React.FC = () => {
  const currentYear = new Date().getFullYear();

  return (
    <Box
      component="footer"
      sx={{
        bgcolor: 'background.paper',
        borderTop: 1,
        borderColor: 'divider',
        py: 6,
        mt: 'auto',
      }}
    >
      <Container maxWidth="lg">
        <Grid container spacing={4}>
          <Grid item xs={12} sm={6} md={3}>
            <Typography variant="h6" color="text.primary" gutterBottom>
              Unit Shop
            </Typography>
            <Typography variant="body2" color="text.secondary">
              Transparent raffle shopping with blockchain-powered fairness and no-loss guarantee.
            </Typography>
            <Box sx={{ mt: 2 }}>
              <IconButton
                color="primary"
                aria-label="Twitter"
                component="a"
                href="https://twitter.com/unitshop"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Twitter />
              </IconButton>
              <IconButton
                color="primary"
                aria-label="Facebook"
                component="a"
                href="https://facebook.com/unitshop"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Facebook />
              </IconButton>
              <IconButton
                color="primary"
                aria-label="Instagram"
                component="a"
                href="https://instagram.com/unitshop"
                target="_blank"
                rel="noopener noreferrer"
              >
                <Instagram />
              </IconButton>
              <IconButton
                color="primary"
                aria-label="LinkedIn"
                component="a"
                href="https://linkedin.com/company/unitshop"
                target="_blank"
                rel="noopener noreferrer"
              >
                <LinkedIn />
              </IconButton>
              <IconButton
                color="primary"
                aria-label="GitHub"
                component="a"
                href="https://github.com/unitshop"
                target="_blank"
                rel="noopener noreferrer"
              >
                <GitHub />
              </IconButton>
            </Box>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Typography variant="h6" color="text.primary" gutterBottom>
              Platform
            </Typography>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
              <Link href="/raffles" color="text.secondary" underline="hover">
                Browse Raffles
              </Link>
              <Link href="/how-it-works" color="text.secondary" underline="hover">
                How It Works
              </Link>
              <Link href="/sellers" color="text.secondary" underline="hover">
                For Sellers
              </Link>
              <Link href="/pricing" color="text.secondary" underline="hover">
                Pricing
              </Link>
            </Box>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Typography variant="h6" color="text.primary" gutterBottom>
              Support
            </Typography>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
              <Link href="/help" color="text.secondary" underline="hover">
                Help Center
              </Link>
              <Link href="/contact" color="text.secondary" underline="hover">
                Contact Us
              </Link>
              <Link href="/faq" color="text.secondary" underline="hover">
                FAQ
              </Link>
              <Link href="/status" color="text.secondary" underline="hover">
                System Status
              </Link>
            </Box>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Typography variant="h6" color="text.primary" gutterBottom>
              Legal
            </Typography>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
              <Link href="/terms" color="text.secondary" underline="hover">
                Terms of Service
              </Link>
              <Link href="/privacy" color="text.secondary" underline="hover">
                Privacy Policy
              </Link>
              <Link href="/cookies" color="text.secondary" underline="hover">
                Cookie Policy
              </Link>
              <Link href="/compliance" color="text.secondary" underline="hover">
                Compliance
              </Link>
            </Box>
          </Grid>
        </Grid>

        <Divider sx={{ my: 4 }} />

        <Box
          sx={{
            display: 'flex',
            flexDirection: { xs: 'column', sm: 'row' },
            justifyContent: 'space-between',
            alignItems: 'center',
            gap: 2,
          }}
        >
          <Typography variant="body2" color="text.secondary">
            © {currentYear} Unit Shop. All rights reserved.
          </Typography>
          
          <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap' }}>
            <Typography variant="body2" color="text.secondary">
              Built with ❤️ using blockchain technology
            </Typography>
          </Box>
        </Box>
      </Container>
    </Box>
  );
};

export default Footer;