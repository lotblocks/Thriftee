import { renderHook, act } from '@testing-library/react';
import { useTouchGestures, useHapticFeedback, useMobileInteractions } from '../useTouchGestures';
import { createRef } from 'react';

// Mock navigator.vibrate
Object.defineProperty(navigator, 'vibrate', {
  value: jest.fn(),
  writable: true,
});

describe('useTouchGestures', () => {
  let elementRef: React.RefObject<HTMLDivElement>;
  let mockElement: HTMLDivElement;

  beforeEach(() => {
    mockElement = document.createElement('div');
    elementRef = { current: mockElement };
    document.body.appendChild(mockElement);
    jest.clearAllMocks();
  });

  afterEach(() => {
    document.body.removeChild(mockElement);
  });

  it('should initialize without errors', () => {
    const { result } = renderHook(() => useTouchGestures(elementRef));
    expect(result.current.isGesturing).toBe(false);
  });

  it('should detect tap gestures', () => {
    const onTap = jest.fn();
    renderHook(() => useTouchGestures(elementRef, { onTap }));

    // Simulate touch events
    const touchStart = new TouchEvent('touchstart', {
      touches: [{ clientX: 100, clientY: 100 } as Touch],
    });
    const touchEnd = new TouchEvent('touchend', {
      changedTouches: [{ clientX: 100, clientY: 100 } as Touch],
    });

    act(() => {
      mockElement.dispatchEvent(touchStart);
    });

    act(() => {
      mockElement.dispatchEvent(touchEnd);
    });

    expect(onTap).toHaveBeenCalledWith(
      expect.objectContaining({
        x: 100,
        y: 100,
        timestamp: expect.any(Number),
      })
    );
  });

  it('should detect swipe gestures', () => {
    const onSwipe = jest.fn();
    renderHook(() => useTouchGestures(elementRef, { onSwipe, swipeThreshold: 50 }));

    const touchStart = new TouchEvent('touchstart', {
      touches: [{ clientX: 100, clientY: 100 } as Touch],
    });
    const touchEnd = new TouchEvent('touchend', {
      changedTouches: [{ clientX: 200, clientY: 100 } as Touch],
    });

    act(() => {
      mockElement.dispatchEvent(touchStart);
    });

    // Wait a bit to simulate gesture duration
    setTimeout(() => {
      act(() => {
        mockElement.dispatchEvent(touchEnd);
      });
    }, 100);

    expect(onSwipe).toHaveBeenCalledWith(
      expect.objectContaining({
        direction: 'right',
        distance: expect.any(Number),
        velocity: expect.any(Number),
        duration: expect.any(Number),
      })
    );
  });

  it('should detect long press gestures', (done) => {
    const onLongPress = jest.fn();
    renderHook(() => useTouchGestures(elementRef, { onLongPress, longPressDelay: 100 }));

    const touchStart = new TouchEvent('touchstart', {
      touches: [{ clientX: 100, clientY: 100 } as Touch],
    });

    act(() => {
      mockElement.dispatchEvent(touchStart);
    });

    setTimeout(() => {
      expect(onLongPress).toHaveBeenCalledWith(
        expect.objectContaining({
          x: 100,
          y: 100,
          timestamp: expect.any(Number),
        })
      );
      done();
    }, 150);
  });

  it('should detect double tap gestures', () => {
    const onDoubleTap = jest.fn();
    renderHook(() => useTouchGestures(elementRef, { onDoubleTap, doubleTapDelay: 300 }));

    const createTouchEvent = (x: number, y: number) => ({
      touchStart: new TouchEvent('touchstart', {
        touches: [{ clientX: x, clientY: y } as Touch],
      }),
      touchEnd: new TouchEvent('touchend', {
        changedTouches: [{ clientX: x, clientY: y } as Touch],
      }),
    });

    const firstTap = createTouchEvent(100, 100);
    const secondTap = createTouchEvent(105, 105);

    // First tap
    act(() => {
      mockElement.dispatchEvent(firstTap.touchStart);
      mockElement.dispatchEvent(firstTap.touchEnd);
    });

    // Second tap quickly after
    setTimeout(() => {
      act(() => {
        mockElement.dispatchEvent(secondTap.touchStart);
        mockElement.dispatchEvent(secondTap.touchEnd);
      });

      expect(onDoubleTap).toHaveBeenCalledWith(
        expect.objectContaining({
          x: 105,
          y: 105,
          timestamp: expect.any(Number),
        })
      );
    }, 100);
  });

  it('should handle pan gestures', () => {
    const onPan = jest.fn();
    renderHook(() => useTouchGestures(elementRef, { onPan }));

    const touchStart = new TouchEvent('touchstart', {
      touches: [{ clientX: 100, clientY: 100 } as Touch],
    });
    const touchMove = new TouchEvent('touchmove', {
      touches: [{ clientX: 120, clientY: 110 } as Touch],
    });

    act(() => {
      mockElement.dispatchEvent(touchStart);
      mockElement.dispatchEvent(touchMove);
    });

    expect(onPan).toHaveBeenCalledWith({ x: 20, y: 10 });
  });

  it('should handle pinch gestures', () => {
    const onPinch = jest.fn();
    renderHook(() => useTouchGestures(elementRef, { onPinch }));

    const touchStart = new TouchEvent('touchstart', {
      touches: [
        { clientX: 100, clientY: 100 } as Touch,
        { clientX: 200, clientY: 100 } as Touch,
      ],
    });
    const touchMove = new TouchEvent('touchmove', {
      touches: [
        { clientX: 90, clientY: 100 } as Touch,
        { clientX: 210, clientY: 100 } as Touch,
      ],
    });

    act(() => {
      mockElement.dispatchEvent(touchStart);
      mockElement.dispatchEvent(touchMove);
    });

    expect(onPinch).toHaveBeenCalledWith(
      expect.objectContaining({
        scale: expect.any(Number),
        center: expect.objectContaining({
          x: expect.any(Number),
          y: expect.any(Number),
        }),
      })
    );
  });
});

describe('useHapticFeedback', () => {
  beforeEach(() => {
    jest.clearAllMocks();
  });

  it('should provide haptic feedback functions', () => {
    const { result } = renderHook(() => useHapticFeedback());

    expect(typeof result.current.vibrate).toBe('function');
    expect(typeof result.current.lightTap).toBe('function');
    expect(typeof result.current.mediumTap).toBe('function');
    expect(typeof result.current.heavyTap).toBe('function');
    expect(typeof result.current.doubleTap).toBe('function');
    expect(typeof result.current.success).toBe('function');
    expect(typeof result.current.error).toBe('function');
  });

  it('should call navigator.vibrate with correct patterns', () => {
    const { result } = renderHook(() => useHapticFeedback());

    act(() => {
      result.current.lightTap();
    });
    expect(navigator.vibrate).toHaveBeenCalledWith(50);

    act(() => {
      result.current.mediumTap();
    });
    expect(navigator.vibrate).toHaveBeenCalledWith(100);

    act(() => {
      result.current.heavyTap();
    });
    expect(navigator.vibrate).toHaveBeenCalledWith(200);

    act(() => {
      result.current.doubleTap();
    });
    expect(navigator.vibrate).toHaveBeenCalledWith([50, 50, 50]);

    act(() => {
      result.current.success();
    });
    expect(navigator.vibrate).toHaveBeenCalledWith([100, 50, 100]);

    act(() => {
      result.current.error();
    });
    expect(navigator.vibrate).toHaveBeenCalledWith([200, 100, 200, 100, 200]);
  });

  it('should handle custom vibration patterns', () => {
    const { result } = renderHook(() => useHapticFeedback());

    act(() => {
      result.current.vibrate([100, 200, 300]);
    });
    expect(navigator.vibrate).toHaveBeenCalledWith([100, 200, 300]);

    act(() => {
      result.current.vibrate(500);
    });
    expect(navigator.vibrate).toHaveBeenCalledWith(500);
  });
});

describe('useMobileInteractions', () => {
  beforeEach(() => {
    // Mock window.scrollY
    Object.defineProperty(window, 'scrollY', {
      value: 0,
      writable: true,
    });
  });

  it('should track scroll state', () => {
    const { result } = renderHook(() => useMobileInteractions());

    expect(result.current.isScrolling).toBe(false);
    expect(result.current.scrollDirection).toBe(null);
  });

  it('should detect scroll direction', (done) => {
    const { result } = renderHook(() => useMobileInteractions());

    // Simulate scroll down
    Object.defineProperty(window, 'scrollY', { value: 100 });
    
    act(() => {
      window.dispatchEvent(new Event('scroll'));
    });

    setTimeout(() => {
      expect(result.current.isScrolling).toBe(true);
      expect(result.current.scrollDirection).toBe('down');
      done();
    }, 50);
  });
});