import React, { useState, useRef } from 'react';
import {
  Box,
  Stack,
  TextField,
  Button,
  Avatar,
  Typography,
  Grid,
  Alert,
  IconButton,
  Tooltip,
  Card,
  CardContent,
  Chip,
} from '@mui/material';
import {
  PhotoCamera,
  Save,
  Cancel,
  Edit,
  Verified,
} from '@mui/icons-material';
import { useForm, Controller } from 'react-hook-form';
import { toast } from 'react-toastify';

import { useAppSelector, useAppDispatch } from '../../store';
import { updateUser } from '../../store/slices/authSlice';
import { authService } from '../../services/authService';

interface ProfileFormData {
  firstName: string;
  lastName: string;
  email: string;
  phone: string;
  dateOfBirth: string;
  address: {
    street: string;
    city: string;
    state: string;
    zipCode: string;
    country: string;
  };
}

const ProfileEditor: React.FC = () => {
  const dispatch = useAppDispatch();
  const { user } = useAppSelector(state => state.auth);
  const [isEditing, setIsEditing] = useState(false);
  const [isLoading, setIsLoading] = useState(false);
  const [avatarFile, setAvatarFile] = useState<File | null>(null);
  const [avatarPreview, setAvatarPreview] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const {
    control,
    handleSubmit,
    reset,
    formState: { errors, isDirty },
  } = useForm<ProfileFormData>({
    defaultValues: {
      firstName: user?.profile?.firstName || '',
      lastName: user?.profile?.lastName || '',
      email: user?.email || '',
      phone: user?.profile?.phone || '',
      dateOfBirth: user?.profile?.dateOfBirth || '',
      address: {
        street: user?.profile?.address?.street || '',
        city: user?.profile?.address?.city || '',
        state: user?.profile?.address?.state || '',
        zipCode: user?.profile?.address?.zipCode || '',
        country: user?.profile?.address?.country || 'US',
      },
    },
  });

  const handleAvatarChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      if (file.size > 5 * 1024 * 1024) { // 5MB limit
        toast.error('Image size must be less than 5MB');
        return;
      }

      setAvatarFile(file);
      const reader = new FileReader();
      reader.onload = (e) => {
        setAvatarPreview(e.target?.result as string);
      };
      reader.readAsDataURL(file);
    }
  };

  const handleAvatarUpload = async () => {
    if (!avatarFile) return null;

    try {
      const response = await authService.uploadAvatar(avatarFile);
      return response.avatarUrl;
    } catch (error) {
      console.error('Avatar upload failed:', error);
      toast.error('Failed to upload avatar');
      return null;
    }
  };

  const onSubmit = async (data: ProfileFormData) => {
    setIsLoading(true);
    try {
      let avatarUrl = user?.profile?.avatar;

      // Upload avatar if changed
      if (avatarFile) {
        const uploadedUrl = await handleAvatarUpload();
        if (uploadedUrl) {
          avatarUrl = uploadedUrl;
        }
      }

      // Update profile
      const updatedUser = await authService.updateProfile({
        ...user,
        email: data.email,
        profile: {
          ...user?.profile,
          firstName: data.firstName,
          lastName: data.lastName,
          phone: data.phone,
          dateOfBirth: data.dateOfBirth,
          avatar: avatarUrl,
          address: data.address,
        },
      });

      dispatch(updateUser(updatedUser));
      setIsEditing(false);
      setAvatarFile(null);
      setAvatarPreview(null);
      toast.success('Profile updated successfully!');
    } catch (error: any) {
      console.error('Profile update failed:', error);
      toast.error(error.response?.data?.message || 'Failed to update profile');
    } finally {
      setIsLoading(false);
    }
  };

  const handleCancel = () => {
    reset();
    setIsEditing(false);
    setAvatarFile(null);
    setAvatarPreview(null);
  };

  const currentAvatar = avatarPreview || user?.profile?.avatar;

  return (
    <Box>
      <Stack spacing={4}>
        {/* Profile header */}
        <Card>
          <CardContent>
            <Stack direction="row" spacing={3} alignItems="center">
              <Box sx={{ position: 'relative' }}>
                <Avatar
                  src={currentAvatar}
                  sx={{ width: 100, height: 100, fontSize: '2rem' }}
                >
                  {user?.profile?.firstName?.[0] || user?.email?.[0]?.toUpperCase()}
                </Avatar>
                {isEditing && (
                  <IconButton
                    sx={{
                      position: 'absolute',
                      bottom: -5,
                      right: -5,
                      backgroundColor: 'primary.main',
                      color: 'white',
                      '&:hover': {
                        backgroundColor: 'primary.dark',
                      },
                    }}
                    size="small"
                    onClick={() => fileInputRef.current?.click()}
                  >
                    <PhotoCamera sx={{ fontSize: 16 }} />
                  </IconButton>
                )}
                <input
                  ref={fileInputRef}
                  type="file"
                  accept="image/*"
                  onChange={handleAvatarChange}
                  style={{ display: 'none' }}
                />
              </Box>

              <Box sx={{ flex: 1 }}>
                <Stack direction="row" alignItems="center" spacing={1} mb={1}>
                  <Typography variant="h5" fontWeight={600}>
                    {user?.profile?.firstName && user?.profile?.lastName
                      ? `${user.profile.firstName} ${user.profile.lastName}`
                      : user?.email?.split('@')[0] || 'User'
                    }
                  </Typography>
                  <Chip
                    icon={<Verified />}
                    label="Verified"
                    color="success"
                    size="small"
                  />
                </Stack>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  {user?.email}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Member since {new Date(user?.createdAt || '').toLocaleDateString()}
                </Typography>
              </Box>

              <Box>
                {!isEditing ? (
                  <Button
                    variant="outlined"
                    startIcon={<Edit />}
                    onClick={() => setIsEditing(true)}
                  >
                    Edit Profile
                  </Button>
                ) : (
                  <Stack direction="row" spacing={1}>
                    <Button
                      variant="outlined"
                      startIcon={<Cancel />}
                      onClick={handleCancel}
                      disabled={isLoading}
                    >
                      Cancel
                    </Button>
                    <Button
                      variant="contained"
                      startIcon={<Save />}
                      onClick={handleSubmit(onSubmit)}
                      disabled={isLoading || !isDirty}
                      sx={{
                        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
                        '&:hover': {
                          background: 'linear-gradient(135deg, #5a6fd8 0%, #6a4190 100%)',
                        },
                      }}
                    >
                      {isLoading ? 'Saving...' : 'Save Changes'}
                    </Button>
                  </Stack>
                )}
              </Box>
            </Stack>
          </CardContent>
        </Card>

        {/* Profile form */}
        <form onSubmit={handleSubmit(onSubmit)}>
          <Stack spacing={4}>
            {/* Personal Information */}
            <Box>
              <Typography variant="h6" gutterBottom fontWeight={600}>
                Personal Information
              </Typography>
              <Grid container spacing={3}>
                <Grid item xs={12} sm={6}>
                  <Controller
                    name="firstName"
                    control={control}
                    rules={{ required: 'First name is required' }}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="First Name"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.firstName}
                        helperText={errors.firstName?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <Controller
                    name="lastName"
                    control={control}
                    rules={{ required: 'Last name is required' }}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="Last Name"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.lastName}
                        helperText={errors.lastName?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <Controller
                    name="email"
                    control={control}
                    rules={{
                      required: 'Email is required',
                      pattern: {
                        value: /^[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}$/i,
                        message: 'Invalid email address',
                      },
                    }}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="Email Address"
                        type="email"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.email}
                        helperText={errors.email?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <Controller
                    name="phone"
                    control={control}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="Phone Number"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.phone}
                        helperText={errors.phone?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <Controller
                    name="dateOfBirth"
                    control={control}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="Date of Birth"
                        type="date"
                        fullWidth
                        disabled={!isEditing}
                        InputLabelProps={{ shrink: true }}
                        error={!!errors.dateOfBirth}
                        helperText={errors.dateOfBirth?.message}
                      />
                    )}
                  />
                </Grid>
              </Grid>
            </Box>

            {/* Address Information */}
            <Box>
              <Typography variant="h6" gutterBottom fontWeight={600}>
                Address Information
              </Typography>
              <Grid container spacing={3}>
                <Grid item xs={12}>
                  <Controller
                    name="address.street"
                    control={control}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="Street Address"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.address?.street}
                        helperText={errors.address?.street?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <Controller
                    name="address.city"
                    control={control}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="City"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.address?.city}
                        helperText={errors.address?.city?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={3}>
                  <Controller
                    name="address.state"
                    control={control}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="State"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.address?.state}
                        helperText={errors.address?.state?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={3}>
                  <Controller
                    name="address.zipCode"
                    control={control}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="ZIP Code"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.address?.zipCode}
                        helperText={errors.address?.zipCode?.message}
                      />
                    )}
                  />
                </Grid>
                <Grid item xs={12} sm={6}>
                  <Controller
                    name="address.country"
                    control={control}
                    render={({ field }) => (
                      <TextField
                        {...field}
                        label="Country"
                        fullWidth
                        disabled={!isEditing}
                        error={!!errors.address?.country}
                        helperText={errors.address?.country?.message}
                      />
                    )}
                  />
                </Grid>
              </Grid>
            </Box>

            {isEditing && isDirty && (
              <Alert severity="info">
                You have unsaved changes. Don't forget to save your profile!
              </Alert>
            )}
          </Stack>
        </form>
      </Stack>
    </Box>
  );
};

export default ProfileEditor;