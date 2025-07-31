import React from 'react';
import {
  Box,
  Typography,
  alpha,
} from '@mui/material';
import {
  CheckCircle,
  RadioButtonUnchecked,
  Lock,
} from '@mui/icons-material';

interface MobileGridCellProps {
  boxNumber: number;
  isSold: boolean;
  isSelected: boolean;
  isRecentlyPurchased?: boolean;
  onClick: () => void;
  disabled: boolean;
  size?: 'small' | 'medium' | 'large';
}

const MobileGridCell: React.FC<MobileGridCellProps> = ({
  boxNumber,
  isSold,
  isSelected,
  isRecentlyPurchased = false,
  onClick,
  disabled,
  size = 'medium',
}) => {
  // Get size-specific dimensions
  const getSizeStyles = () => {
    switch (size) {
      case 'small':
        return {
          minHeight: '32px',
          minWidth: '32px',
          fontSize: '0.6rem',
          iconSize: 12,
        };
      case 'large':
        return {
          minHeight: '56px',
          minWidth: '56px',
          fontSize: '0.8rem',
          iconSize: 20,
        };
      default:
        return {
          minHeight: '44px',
          minWidth: '44px',
          fontSize: '0.7rem',
          iconSize: 16,
        };
    }
  };

  const sizeStyles = getSizeStyles();

  // Determine cell state and styling
  const getCellState = () => {
    if (isSold) return 'sold';
    if (isSelected) return 'selected';
    return 'available';
  };

  const cellState = getCellState();

  // Get styling based on state with mobile optimizations
  const getCellStyles = () => {
    const baseStyles = {
      position: 'relative' as const,
      aspectRatio: '1',
      border: '2px solid',
      borderRadius: 1,
      cursor: disabled ? 'not-allowed' : 'pointer',
      transition: 'all 0.2s ease-in-out',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      overflow: 'hidden',
      minHeight: sizeStyles.minHeight,
      minWidth: sizeStyles.minWidth,
      // Mobile-specific optimizations
      touchAction: 'manipulation',
      userSelect: 'none',
      WebkitTapHighlightColor: 'transparent',
      // Larger touch target for better mobile UX
      '&::before': {
        content: '""',
        position: 'absolute',
        top: '-4px',
        left: '-4px',
        right: '-4px',
        bottom: '-4px',
        zIndex: -1,
      },
    };

    switch (cellState) {
      case 'sold':
        return {
          ...baseStyles,
          backgroundColor: alpha('#f44336', 0.1),
          borderColor: '#f44336',
          cursor: 'not-allowed',
        };
      case 'selected':
        return {
          ...baseStyles,
          backgroundColor: alpha('#667eea', 0.2),
          borderColor: '#667eea',
          transform: 'scale(0.95)',
          boxShadow: '0 0 0 2px rgba(102, 126, 234, 0.3)',
        };
      default:
        return {
          ...baseStyles,
          backgroundColor: 'background.paper',
          borderColor: 'divider',
          '&:active': {
            transform: 'scale(0.95)',
            backgroundColor: alpha('#667eea', 0.1),
          },
        };
    }
  };

  // Get icon based on state with mobile-appropriate sizes
  const getIcon = () => {
    const iconProps = { sx: { fontSize: sizeStyles.iconSize } };
    
    if (isSold) {
      return <Lock {...iconProps} sx={{ ...iconProps.sx, color: 'error.main' }} />;
    }
    if (isSelected) {
      return <CheckCircle {...iconProps} sx={{ ...iconProps.sx, color: 'primary.main' }} />;
    }
    return <RadioButtonUnchecked {...iconProps} sx={{ ...iconProps.sx, color: 'text.secondary' }} />;
  };

  return (
    <Box
      sx={getCellStyles()}
      onClick={onClick}
      role="button"
      aria-label={`Box ${boxNumber}${isSold ? ' - sold' : isSelected ? ' - selected' : ' - available'}`}
      tabIndex={disabled ? -1 : 0}
      onKeyDown={(e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          onClick();
        }
      }}
    >
      {/* Overlay content */}
      <Box
        sx={{
          position: 'relative',
          zIndex: 1,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          gap: 0.25,
        }}
      >
        {getIcon()}
        <Typography
          variant="caption"
          sx={{
            fontSize: sizeStyles.fontSize,
            fontWeight: 600,
            color: isSold ? 'error.main' : isSelected ? 'primary.main' : 'text.secondary',
            lineHeight: 1,
          }}
        >
          {boxNumber}
        </Typography>
      </Box>

      {/* Selection indicator */}
      {isSelected && (
        <Box
          sx={{
            position: 'absolute',
            top: -2,
            right: -2,
            width: 10,
            height: 10,
            backgroundColor: 'primary.main',
            borderRadius: '50%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 2,
          }}
        >
          <CheckCircle sx={{ fontSize: 6, color: 'white' }} />
        </Box>
      )}

      {/* Sold indicator */}
      {isSold && (
        <Box
          sx={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: alpha('#f44336', 0.8),
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 2,
          }}
        >
          <Typography
            variant="caption"
            sx={{
              color: 'white',
              fontWeight: 700,
              fontSize: '0.5rem',
              transform: 'rotate(-45deg)',
              lineHeight: 1,
            }}
          >
            SOLD
          </Typography>
        </Box>
      )}

      {/* Recently purchased animation */}
      {isRecentlyPurchased && (
        <Box
          sx={{
            position: 'absolute',
            top: -2,
            left: -2,
            right: -2,
            bottom: -2,
            border: '2px solid',
            borderColor: 'success.main',
            borderRadius: 1,
            zIndex: 3,
            animation: 'mobilePulse 1s ease-in-out 3',
            '@keyframes mobilePulse': {
              '0%': {
                opacity: 1,
                transform: 'scale(1)',
              },
              '50%': {
                opacity: 0.7,
                transform: 'scale(1.1)',
              },
              '100%': {
                opacity: 1,
                transform: 'scale(1)',
              },
            },
          }}
        />
      )}

      {/* Touch feedback overlay */}
      <Box
        sx={{
          position: 'absolute',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          backgroundColor: 'transparent',
          zIndex: 1,
          pointerEvents: 'none',
          transition: 'background-color 0.1s ease',
          '&:active': {
            backgroundColor: alpha('#667eea', 0.1),
          },
        }}
      />
    </Box>
  );
};

export default MobileGridCell;