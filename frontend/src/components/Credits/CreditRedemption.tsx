import React, { useState } from 'react';
import {
  Box,
  Stack,
  Typography,
  Card,
  CardContent,
  CardMedia,
  Grid,
  Button,
  Chip,
  TextField,
  InputAdornment,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Alert,
  Divider,
  IconButton,
  Tooltip,
} from '@mui/material';
import {
  LocalOffer,
  Search,
  FilterList,
  ShoppingCart,
  Star,
  Favorite,
  FavoriteBorder,
  Close,
} from '@mui/icons-material';
import { toast } from 'react-toastify';

import { useAppSelector } from '../../store';

interface RedeemableItem {
  id: string;
  title: string;
  description: string;
  imageUrl: string;
  creditCost: number;
  originalPrice: number;
  category: string;
  rating: number;
  reviewCount: number;
  inStock: boolean;
  featured: boolean;
  tags: string[];
}

const CreditRedemption: React.FC = () => {
  const { balance } = useAppSelector(state => state.credit);
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [selectedItem, setSelectedItem] = useState<RedeemableItem | null>(null);
  const [showRedemptionDialog, setShowRedemptionDialog] = useState(false);
  const [favorites, setFavorites] = useState<Set<string>>(new Set());

  // Mock redeemable items - in real app, this would come from API
  const redeemableItems: RedeemableItem[] = [
    {
      id: '1',
      title: 'Wireless Bluetooth Earbuds',
      description: 'High-quality wireless earbuds with noise cancellation',
      imageUrl: 'https://images.unsplash.com/photo-1606220945770-b5b6c2c55bf1?w=300',
      creditCost: 25,
      originalPrice: 49.99,
      category: 'Electronics',
      rating: 4.5,
      reviewCount: 128,
      inStock: true,
      featured: true,
      tags: ['Popular', 'New'],
    },
    {
      id: '2',
      title: 'Premium Coffee Beans (1lb)',
      description: 'Artisan roasted coffee beans from sustainable farms',
      imageUrl: 'https://images.unsplash.com/photo-1559056199-641a0ac8b55e?w=300',
      creditCost: 15,
      originalPrice: 24.99,
      category: 'Food & Beverage',
      rating: 4.8,
      reviewCount: 89,
      inStock: true,
      featured: false,
      tags: ['Organic', 'Fair Trade'],
    },
    {
      id: '3',
      title: 'Smartphone Phone Case',
      description: 'Protective case with wireless charging compatibility',
      imageUrl: 'https://images.unsplash.com/photo-1556656793-08538906a9f8?w=300',
      creditCost: 10,
      originalPrice: 19.99,
      category: 'Accessories',
      rating: 4.2,
      reviewCount: 67,
      inStock: true,
      featured: false,
      tags: ['Protective'],
    },
    {
      id: '4',
      title: 'Fitness Tracker Band',
      description: 'Track your daily activity and health metrics',
      imageUrl: 'https://images.unsplash.com/photo-1575311373937-040b8e1fd5b6?w=300',
      creditCost: 35,
      originalPrice: 79.99,
      category: 'Health & Fitness',
      rating: 4.6,
      reviewCount: 203,
      inStock: false,
      featured: true,
      tags: ['Health', 'Popular'],
    },
    {
      id: '5',
      title: 'Eco-Friendly Water Bottle',
      description: 'Stainless steel water bottle with temperature control',
      imageUrl: 'https://images.unsplash.com/photo-1602143407151-7111542de6e8?w=300',
      creditCost: 12,
      originalPrice: 29.99,
      category: 'Lifestyle',
      rating: 4.4,
      reviewCount: 156,
      inStock: true,
      featured: false,
      tags: ['Eco-Friendly', 'Sustainable'],
    },
    {
      id: '6',
      title: 'Gaming Mouse Pad',
      description: 'Large gaming mouse pad with RGB lighting',
      imageUrl: 'https://images.unsplash.com/photo-1527814050087-3793815479db?w=300',
      creditCost: 8,
      originalPrice: 15.99,
      category: 'Gaming',
      rating: 4.3,
      reviewCount: 94,
      inStock: true,
      featured: false,
      tags: ['Gaming', 'RGB'],
    },
  ];

  const categories = ['all', ...Array.from(new Set(redeemableItems.map(item => item.category)))];

  // Filter items based on search and category
  const filteredItems = redeemableItems.filter(item => {
    const matchesSearch = item.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         item.description.toLowerCase().includes(searchTerm.toLowerCase()) ||
                         item.tags.some(tag => tag.toLowerCase().includes(searchTerm.toLowerCase()));
    
    const matchesCategory = selectedCategory === 'all' || item.category === selectedCategory;
    
    return matchesSearch && matchesCategory;
  });

  const handleRedeemItem = (item: RedeemableItem) => {
    setSelectedItem(item);
    setShowRedemptionDialog(true);
  };

  const confirmRedemption = async () => {
    if (!selectedItem) return;

    if (balance < selectedItem.creditCost) {
      toast.error('Insufficient credits for this redemption');
      return;
    }

    try {
      // Simulate API call
      await new Promise(resolve => setTimeout(resolve, 1000));
      
      toast.success(`Successfully redeemed ${selectedItem.title}!`);
      setShowRedemptionDialog(false);
      setSelectedItem(null);
    } catch (error) {
      toast.error('Failed to redeem item. Please try again.');
    }
  };

  const toggleFavorite = (itemId: string) => {
    setFavorites(prev => {
      const newFavorites = new Set(prev);
      if (newFavorites.has(itemId)) {
        newFavorites.delete(itemId);
      } else {
        newFavorites.add(itemId);
      }
      return newFavorites;
    });
  };

  const getSavingsPercentage = (item: RedeemableItem) => {
    return Math.round(((item.originalPrice - item.creditCost) / item.originalPrice) * 100);
  };

  return (
    <Box>
      <Stack spacing={4}>
        {/* Header */}
        <Box>
          <Stack direction="row" alignItems="center" spacing={2} mb={2}>
            <LocalOffer color="primary" sx={{ fontSize: 32 }} />
            <Box>
              <Typography variant="h5" fontWeight={600}>
                Redeem Credits
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Use your credits to get free items and exclusive rewards
              </Typography>
            </Box>
          </Stack>

          {/* Current balance */}
          <Card variant="outlined" sx={{ mb: 3 }}>
            <CardContent>
              <Stack direction="row" justifyContent="space-between" alignItems="center">
                <Typography variant="body1" color="text.secondary">
                  Available Credits
                </Typography>
                <Typography variant="h5" fontWeight={600} color="primary">
                  ${balance.toFixed(2)}
                </Typography>
              </Stack>
            </CardContent>
          </Card>
        </Box>

        {/* Search and filters */}
        <Card>
          <CardContent>
            <Stack direction="row" spacing={2} alignItems="center" flexWrap="wrap">
              <TextField
                placeholder="Search items..."
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

              <Stack direction="row" spacing={1} flexWrap="wrap">
                {categories.map((category) => (
                  <Chip
                    key={category}
                    label={category === 'all' ? 'All Categories' : category}
                    onClick={() => setSelectedCategory(category)}
                    color={selectedCategory === category ? 'primary' : 'default'}
                    variant={selectedCategory === category ? 'filled' : 'outlined'}
                  />
                ))}
              </Stack>
            </Stack>
          </CardContent>
        </Card>

        {/* Featured items */}
        {filteredItems.some(item => item.featured) && (
          <Box>
            <Typography variant="h6" gutterBottom fontWeight={600}>
              Featured Items
            </Typography>
            <Grid container spacing={3}>
              {filteredItems
                .filter(item => item.featured)
                .slice(0, 3)
                .map((item) => (
                  <Grid item xs={12} sm={6} md={4} key={item.id}>
                    <Card
                      sx={{
                        height: '100%',
                        position: 'relative',
                        transition: 'transform 0.2s',
                        '&:hover': {
                          transform: 'translateY(-4px)',
                          boxShadow: 4,
                        },
                      }}
                    >
                      <Box sx={{ position: 'relative' }}>
                        <CardMedia
                          component="img"
                          height="200"
                          image={item.imageUrl}
                          alt={item.title}
                        />
                        <Chip
                          label="Featured"
                          color="primary"
                          size="small"
                          icon={<Star />}
                          sx={{
                            position: 'absolute',
                            top: 8,
                            left: 8,
                            fontWeight: 600,
                          }}
                        />
                        <IconButton
                          sx={{
                            position: 'absolute',
                            top: 8,
                            right: 8,
                            backgroundColor: 'rgba(255, 255, 255, 0.9)',
                            '&:hover': {
                              backgroundColor: 'rgba(255, 255, 255, 1)',
                            },
                          }}
                          onClick={() => toggleFavorite(item.id)}
                        >
                          {favorites.has(item.id) ? (
                            <Favorite color="error" />
                          ) : (
                            <FavoriteBorder />
                          )}
                        </IconButton>
                      </Box>
                      
                      <CardContent>
                        <Stack spacing={2}>
                          <Box>
                            <Typography variant="h6" fontWeight={600} noWrap>
                              {item.title}
                            </Typography>
                            <Typography variant="body2" color="text.secondary" noWrap>
                              {item.description}
                            </Typography>
                          </Box>

                          <Stack direction="row" alignItems="center" spacing={1}>
                            <Typography variant="body2" color="text.secondary">
                              ★ {item.rating}
                            </Typography>
                            <Typography variant="body2" color="text.secondary">
                              ({item.reviewCount} reviews)
                            </Typography>
                          </Stack>

                          <Stack direction="row" spacing={1} flexWrap="wrap">
                            {item.tags.slice(0, 2).map((tag) => (
                              <Chip
                                key={tag}
                                label={tag}
                                size="small"
                                variant="outlined"
                                sx={{ fontSize: '0.7rem' }}
                              />
                            ))}
                          </Stack>

                          <Divider />

                          <Stack direction="row" justifyContent="space-between" alignItems="center">
                            <Box>
                              <Typography variant="h6" fontWeight={600} color="primary">
                                ${item.creditCost} credits
                              </Typography>
                              <Typography variant="caption" color="text.secondary">
                                Save {getSavingsPercentage(item)}% (${item.originalPrice})
                              </Typography>
                            </Box>
                            
                            <Button
                              variant="contained"
                              size="small"
                              onClick={() => handleRedeemItem(item)}
                              disabled={!item.inStock || balance < item.creditCost}
                              sx={{
                                background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                                '&:hover': {
                                  background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                                },
                              }}
                            >
                              {!item.inStock ? 'Out of Stock' : 'Redeem'}
                            </Button>
                          </Stack>
                        </Stack>
                      </CardContent>
                    </Card>
                  </Grid>
                ))}
            </Grid>
          </Box>
        )}

        {/* All items */}
        <Box>
          <Typography variant="h6" gutterBottom fontWeight={600}>
            All Items ({filteredItems.length})
          </Typography>
          
          <Grid container spacing={3}>
            {filteredItems.map((item) => (
              <Grid item xs={12} sm={6} md={4} lg={3} key={item.id}>
                <Card
                  sx={{
                    height: '100%',
                    position: 'relative',
                    transition: 'transform 0.2s',
                    '&:hover': {
                      transform: 'translateY(-2px)',
                      boxShadow: 2,
                    },
                  }}
                >
                  <Box sx={{ position: 'relative' }}>
                    <CardMedia
                      component="img"
                      height="160"
                      image={item.imageUrl}
                      alt={item.title}
                    />
                    {!item.inStock && (
                      <Box
                        sx={{
                          position: 'absolute',
                          top: 0,
                          left: 0,
                          right: 0,
                          bottom: 0,
                          backgroundColor: 'rgba(0, 0, 0, 0.6)',
                          display: 'flex',
                          alignItems: 'center',
                          justifyContent: 'center',
                        }}
                      >
                        <Typography variant="h6" color="white" fontWeight={600}>
                          Out of Stock
                        </Typography>
                      </Box>
                    )}
                    <IconButton
                      sx={{
                        position: 'absolute',
                        top: 8,
                        right: 8,
                        backgroundColor: 'rgba(255, 255, 255, 0.9)',
                        '&:hover': {
                          backgroundColor: 'rgba(255, 255, 255, 1)',
                        },
                      }}
                      onClick={() => toggleFavorite(item.id)}
                    >
                      {favorites.has(item.id) ? (
                        <Favorite color="error" />
                      ) : (
                        <FavoriteBorder />
                      )}
                    </IconButton>
                  </Box>
                  
                  <CardContent sx={{ p: 2 }}>
                    <Stack spacing={1}>
                      <Typography variant="subtitle1" fontWeight={600} noWrap>
                        {item.title}
                      </Typography>
                      
                      <Typography variant="caption" color="text.secondary">
                        ★ {item.rating} ({item.reviewCount})
                      </Typography>

                      <Stack direction="row" justifyContent="space-between" alignItems="center">
                        <Typography variant="body1" fontWeight={600} color="primary">
                          ${item.creditCost}
                        </Typography>
                        
                        <Button
                          variant="outlined"
                          size="small"
                          onClick={() => handleRedeemItem(item)}
                          disabled={!item.inStock || balance < item.creditCost}
                        >
                          Redeem
                        </Button>
                      </Stack>
                    </Stack>
                  </CardContent>
                </Card>
              </Grid>
            ))}
          </Grid>

          {filteredItems.length === 0 && (
            <Box sx={{ textAlign: 'center', py: 6 }}>
              <LocalOffer sx={{ fontSize: 64, color: 'text.secondary', mb: 2 }} />
              <Typography variant="h6" gutterBottom>
                No items found
              </Typography>
              <Typography variant="body2" color="text.secondary">
                Try adjusting your search or category filters
              </Typography>
            </Box>
          )}
        </Box>
      </Stack>

      {/* Redemption confirmation dialog */}
      <Dialog
        open={showRedemptionDialog}
        onClose={() => setShowRedemptionDialog(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>
          <Stack direction="row" alignItems="center" justifyContent="space-between">
            <Typography variant="h6">Confirm Redemption</Typography>
            <IconButton onClick={() => setShowRedemptionDialog(false)}>
              <Close />
            </IconButton>
          </Stack>
        </DialogTitle>
        
        <DialogContent>
          {selectedItem && (
            <Stack spacing={3}>
              <Box display="flex" alignItems="center" spacing={2}>
                <Box
                  component="img"
                  src={selectedItem.imageUrl}
                  alt={selectedItem.title}
                  sx={{ width: 80, height: 80, borderRadius: 1, objectFit: 'cover' }}
                />
                <Box sx={{ ml: 2 }}>
                  <Typography variant="h6" fontWeight={600}>
                    {selectedItem.title}
                  </Typography>
                  <Typography variant="body2" color="text.secondary">
                    {selectedItem.description}
                  </Typography>
                </Box>
              </Box>

              <Card variant="outlined">
                <CardContent>
                  <Stack spacing={1}>
                    <Stack direction="row" justifyContent="space-between">
                      <Typography>Credit Cost</Typography>
                      <Typography fontWeight={600}>${selectedItem.creditCost}</Typography>
                    </Stack>
                    <Stack direction="row" justifyContent="space-between">
                      <Typography>Original Price</Typography>
                      <Typography color="text.secondary">${selectedItem.originalPrice}</Typography>
                    </Stack>
                    <Stack direction="row" justifyContent="space-between">
                      <Typography color="success.main">You Save</Typography>
                      <Typography color="success.main" fontWeight={600}>
                        ${(selectedItem.originalPrice - selectedItem.creditCost).toFixed(2)} ({getSavingsPercentage(selectedItem)}%)
                      </Typography>
                    </Stack>
                    <Divider />
                    <Stack direction="row" justifyContent="space-between">
                      <Typography>Current Balance</Typography>
                      <Typography>${balance.toFixed(2)}</Typography>
                    </Stack>
                    <Stack direction="row" justifyContent="space-between">
                      <Typography>After Redemption</Typography>
                      <Typography fontWeight={600}>
                        ${(balance - selectedItem.creditCost).toFixed(2)}
                      </Typography>
                    </Stack>
                  </Stack>
                </CardContent>
              </Card>

              {balance < selectedItem.creditCost && (
                <Alert severity="error">
                  Insufficient credits. You need ${(selectedItem.creditCost - balance).toFixed(2)} more credits.
                </Alert>
              )}

              <Alert severity="info">
                <Typography variant="body2">
                  This item will be shipped to your registered address within 3-5 business days.
                  You will receive a tracking number via email once shipped.
                </Typography>
              </Alert>
            </Stack>
          )}
        </DialogContent>
        
        <DialogActions>
          <Button onClick={() => setShowRedemptionDialog(false)}>
            Cancel
          </Button>
          <Button
            onClick={confirmRedemption}
            variant="contained"
            disabled={!selectedItem || balance < selectedItem.creditCost}
            sx={{
              background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
              '&:hover': {
                background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
              },
            }}
          >
            Confirm Redemption
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default CreditRedemption;