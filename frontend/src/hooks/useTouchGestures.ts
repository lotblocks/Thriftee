import { useState, useEffect, useRef, useCallback } from 'react';

interface TouchPoint {
  x: number;
  y: number;
  timestamp: number;
}

interface SwipeGesture {
  direction: 'up' | 'down' | 'left' | 'right';
  distance: number;
  velocity: number;
  duration: number;
}

interface PinchGesture {
  scale: number;
  center: { x: number; y: number };
}

interface TapGesture {
  x: number;
  y: number;
  timestamp: number;
}

interface TouchGestureOptions {
  onSwipe?: (gesture: SwipeGesture) => void;
  onPinch?: (gesture: PinchGesture) => void;
  onTap?: (gesture: TapGesture) => void;
  onDoubleTap?: (gesture: TapGesture) => void;
  onLongPress?: (gesture: TapGesture) => void;
  onPan?: (delta: { x: number; y: number }) => void;
  swipeThreshold?: number;
  longPressDelay?: number;
  doubleTapDelay?: number;
  preventDefault?: boolean;
}

export const useTouchGestures = (
  elementRef: React.RefObject<HTMLElement>,
  options: TouchGestureOptions = {}
) => {
  const {
    onSwipe,
    onPinch,
    onTap,
    onDoubleTap,
    onLongPress,
    onPan,
    swipeThreshold = 50,
    longPressDelay = 500,
    doubleTapDelay = 300,
    preventDefault = true,
  } = options;

  const [isGesturing, setIsGesturing] = useState(false);
  const touchStartRef = useRef<TouchPoint | null>(null);
  const touchesRef = useRef<TouchList | null>(null);
  const lastTapRef = useRef<TouchPoint | null>(null);
  const longPressTimerRef = useRef<NodeJS.Timeout | null>(null);
  const initialPinchDistanceRef = useRef<number>(0);
  const lastPinchScaleRef = useRef<number>(1);

  // Calculate distance between two touch points
  const getDistance = useCallback((touch1: Touch, touch2: Touch): number => {
    const dx = touch1.clientX - touch2.clientX;
    const dy = touch1.clientY - touch2.clientY;
    return Math.sqrt(dx * dx + dy * dy);
  }, []);

  // Calculate center point between two touches
  const getCenter = useCallback((touch1: Touch, touch2: Touch) => {
    return {
      x: (touch1.clientX + touch2.clientX) / 2,
      y: (touch1.clientY + touch2.clientY) / 2,
    };
  }, []);

  // Handle touch start
  const handleTouchStart = useCallback((e: TouchEvent) => {
    if (preventDefault) {
      e.preventDefault();
    }

    const touch = e.touches[0];
    const touchPoint: TouchPoint = {
      x: touch.clientX,
      y: touch.clientY,
      timestamp: Date.now(),
    };

    touchStartRef.current = touchPoint;
    touchesRef.current = e.touches;
    setIsGesturing(true);

    // Handle pinch gesture initialization
    if (e.touches.length === 2) {
      initialPinchDistanceRef.current = getDistance(e.touches[0], e.touches[1]);
      lastPinchScaleRef.current = 1;
    }

    // Start long press timer
    if (e.touches.length === 1 && onLongPress) {
      longPressTimerRef.current = setTimeout(() => {
        if (touchStartRef.current) {
          onLongPress(touchStartRef.current);
        }
      }, longPressDelay);
    }
  }, [preventDefault, onLongPress, longPressDelay, getDistance]);

  // Handle touch move
  const handleTouchMove = useCallback((e: TouchEvent) => {
    if (preventDefault) {
      e.preventDefault();
    }

    if (!touchStartRef.current) return;

    // Clear long press timer on move
    if (longPressTimerRef.current) {
      clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }

    // Handle pinch gesture
    if (e.touches.length === 2 && onPinch && initialPinchDistanceRef.current > 0) {
      const currentDistance = getDistance(e.touches[0], e.touches[1]);
      const scale = currentDistance / initialPinchDistanceRef.current;
      const center = getCenter(e.touches[0], e.touches[1]);

      if (Math.abs(scale - lastPinchScaleRef.current) > 0.01) {
        onPinch({ scale, center });
        lastPinchScaleRef.current = scale;
      }
      return;
    }

    // Handle pan gesture
    if (e.touches.length === 1 && onPan) {
      const touch = e.touches[0];
      const deltaX = touch.clientX - touchStartRef.current.x;
      const deltaY = touch.clientY - touchStartRef.current.y;
      onPan({ x: deltaX, y: deltaY });
    }
  }, [preventDefault, onPinch, onPan, getDistance, getCenter]);

  // Handle touch end
  const handleTouchEnd = useCallback((e: TouchEvent) => {
    if (preventDefault) {
      e.preventDefault();
    }

    // Clear long press timer
    if (longPressTimerRef.current) {
      clearTimeout(longPressTimerRef.current);
      longPressTimerRef.current = null;
    }

    if (!touchStartRef.current) return;

    const touchEnd = e.changedTouches[0];
    const touchEndPoint: TouchPoint = {
      x: touchEnd.clientX,
      y: touchEnd.clientY,
      timestamp: Date.now(),
    };

    const deltaX = touchEndPoint.x - touchStartRef.current.x;
    const deltaY = touchEndPoint.y - touchStartRef.current.y;
    const distance = Math.sqrt(deltaX * deltaX + deltaY * deltaY);
    const duration = touchEndPoint.timestamp - touchStartRef.current.timestamp;
    const velocity = distance / duration;

    // Handle swipe gesture
    if (distance > swipeThreshold && onSwipe) {
      let direction: SwipeGesture['direction'];
      if (Math.abs(deltaX) > Math.abs(deltaY)) {
        direction = deltaX > 0 ? 'right' : 'left';
      } else {
        direction = deltaY > 0 ? 'down' : 'up';
      }

      onSwipe({
        direction,
        distance,
        velocity,
        duration,
      });
    }
    // Handle tap gestures
    else if (distance < 10 && duration < 300) {
      // Check for double tap
      if (lastTapRef.current && onDoubleTap) {
        const timeSinceLastTap = touchEndPoint.timestamp - lastTapRef.current.timestamp;
        const distanceFromLastTap = Math.sqrt(
          Math.pow(touchEndPoint.x - lastTapRef.current.x, 2) +
          Math.pow(touchEndPoint.y - lastTapRef.current.y, 2)
        );

        if (timeSinceLastTap < doubleTapDelay && distanceFromLastTap < 50) {
          onDoubleTap(touchEndPoint);
          lastTapRef.current = null;
          touchStartRef.current = null;
          setIsGesturing(false);
          return;
        }
      }

      // Single tap
      if (onTap) {
        onTap(touchEndPoint);
      }
      lastTapRef.current = touchEndPoint;
    }

    touchStartRef.current = null;
    touchesRef.current = null;
    initialPinchDistanceRef.current = 0;
    lastPinchScaleRef.current = 1;
    setIsGesturing(false);
  }, [preventDefault, swipeThreshold, doubleTapDelay, onSwipe, onTap, onDoubleTap]);

  // Set up event listeners
  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    element.addEventListener('touchstart', handleTouchStart, { passive: !preventDefault });
    element.addEventListener('touchmove', handleTouchMove, { passive: !preventDefault });
    element.addEventListener('touchend', handleTouchEnd, { passive: !preventDefault });

    return () => {
      element.removeEventListener('touchstart', handleTouchStart);
      element.removeEventListener('touchmove', handleTouchMove);
      element.removeEventListener('touchend', handleTouchEnd);
    };
  }, [handleTouchStart, handleTouchMove, handleTouchEnd, preventDefault]);

  // Cleanup timers on unmount
  useEffect(() => {
    return () => {
      if (longPressTimerRef.current) {
        clearTimeout(longPressTimerRef.current);
      }
    };
  }, []);

  return {
    isGesturing,
  };
};

// Hook for handling specific mobile interactions
export const useMobileInteractions = () => {
  const [isScrolling, setIsScrolling] = useState(false);
  const [scrollDirection, setScrollDirection] = useState<'up' | 'down' | null>(null);
  const lastScrollY = useRef(0);

  useEffect(() => {
    let scrollTimer: NodeJS.Timeout;

    const handleScroll = () => {
      const currentScrollY = window.scrollY;
      setScrollDirection(currentScrollY > lastScrollY.current ? 'down' : 'up');
      lastScrollY.current = currentScrollY;

      setIsScrolling(true);
      clearTimeout(scrollTimer);
      scrollTimer = setTimeout(() => {
        setIsScrolling(false);
        setScrollDirection(null);
      }, 150);
    };

    window.addEventListener('scroll', handleScroll, { passive: true });
    return () => {
      window.removeEventListener('scroll', handleScroll);
      clearTimeout(scrollTimer);
    };
  }, []);

  return {
    isScrolling,
    scrollDirection,
  };
};

// Hook for handling pull-to-refresh
export const usePullToRefresh = (
  onRefresh: () => Promise<void> | void,
  threshold: number = 80
) => {
  const [isPulling, setIsPulling] = useState(false);
  const [pullDistance, setPullDistance] = useState(0);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const { isGesturing } = useTouchGestures(containerRef, {
    onPan: ({ y }) => {
      if (window.scrollY === 0 && y > 0) {
        setIsPulling(true);
        setPullDistance(Math.min(y, threshold * 1.5));
      }
    },
    preventDefault: false,
  });

  useEffect(() => {
    if (!isGesturing && isPulling) {
      if (pullDistance >= threshold) {
        setIsRefreshing(true);
        Promise.resolve(onRefresh()).finally(() => {
          setIsRefreshing(false);
          setIsPulling(false);
          setPullDistance(0);
        });
      } else {
        setIsPulling(false);
        setPullDistance(0);
      }
    }
  }, [isGesturing, isPulling, pullDistance, threshold, onRefresh]);

  return {
    containerRef,
    isPulling,
    pullDistance,
    isRefreshing,
  };
};

// Hook for handling haptic feedback
export const useHapticFeedback = () => {
  const vibrate = useCallback((pattern: number | number[]) => {
    if ('vibrate' in navigator) {
      navigator.vibrate(pattern);
    }
  }, []);

  const lightTap = useCallback(() => vibrate(50), [vibrate]);
  const mediumTap = useCallback(() => vibrate(100), [vibrate]);
  const heavyTap = useCallback(() => vibrate(200), [vibrate]);
  const doubleTap = useCallback(() => vibrate([50, 50, 50]), [vibrate]);
  const success = useCallback(() => vibrate([100, 50, 100]), [vibrate]);
  const error = useCallback(() => vibrate([200, 100, 200, 100, 200]), [vibrate]);

  return {
    vibrate,
    lightTap,
    mediumTap,
    heavyTap,
    doubleTap,
    success,
    error,
  };
};