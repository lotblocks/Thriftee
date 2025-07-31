import React, { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { usePerformanceMonitoring } from '../../services/performanceService';

export interface VirtualScrollItem {
  id: string | number;
  height?: number;
  data: any;
}

export interface VirtualScrollProps<T extends VirtualScrollItem> {
  items: T[];
  itemHeight: number | ((item: T, index: number) => number);
  containerHeight: number;
  renderItem: (item: T, index: number, style: React.CSSProperties) => React.ReactNode;
  overscan?: number;
  className?: string;
  onScroll?: (scrollTop: number, scrollLeft: number) => void;
  onItemsRendered?: (startIndex: number, endIndex: number, visibleItems: T[]) => void;
  loadMoreItems?: (startIndex: number, stopIndex: number) => Promise<void>;
  hasNextPage?: boolean;
  isLoading?: boolean;
  threshold?: number;
  estimatedItemHeight?: number;
  scrollToIndex?: number;
  scrollToAlignment?: 'start' | 'center' | 'end' | 'auto';
}

interface ItemPosition {
  index: number;
  top: number;
  height: number;
}

export function VirtualScrollList<T extends VirtualScrollItem>({
  items,
  itemHeight,
  containerHeight,
  renderItem,
  overscan = 5,
  className = '',
  onScroll,
  onItemsRendered,
  loadMoreItems,
  hasNextPage = false,
  isLoading = false,
  threshold = 5,
  estimatedItemHeight = 50,
  scrollToIndex,
  scrollToAlignment = 'auto',
}: VirtualScrollProps<T>) {
  const { startMeasure, endMeasure } = usePerformanceMonitoring();
  const containerRef = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const [isScrolling, setIsScrolling] = useState(false);
  const scrollingTimeoutRef = useRef<NodeJS.Timeout>();
  const itemPositionsRef = useRef<ItemPosition[]>([]);
  const measuredHeightsRef = useRef<Map<number, number>>(new Map());

  // Calculate item positions
  const itemPositions = useMemo(() => {
    startMeasure('virtual-scroll-calculate-positions');
    
    const positions: ItemPosition[] = [];
    let top = 0;

    for (let i = 0; i < items.length; i++) {
      const item = items[i];
      let height: number;

      if (typeof itemHeight === 'function') {
        // Use measured height if available, otherwise use function result
        height = measuredHeightsRef.current.get(i) || itemHeight(item, i);
      } else {
        height = measuredHeightsRef.current.get(i) || itemHeight;
      }

      positions.push({
        index: i,
        top,
        height,
      });

      top += height;
    }

    itemPositionsRef.current = positions;
    endMeasure('virtual-scroll-calculate-positions');
    return positions;
  }, [items, itemHeight, startMeasure, endMeasure]);

  // Calculate total height
  const totalHeight = useMemo(() => {
    if (itemPositions.length === 0) return 0;
    const lastItem = itemPositions[itemPositions.length - 1];
    return lastItem.top + lastItem.height;
  }, [itemPositions]);

  // Find visible range
  const visibleRange = useMemo(() => {
    startMeasure('virtual-scroll-calculate-range');
    
    if (itemPositions.length === 0) {
      endMeasure('virtual-scroll-calculate-range');
      return { startIndex: 0, endIndex: 0 };
    }

    const containerTop = scrollTop;
    const containerBottom = scrollTop + containerHeight;

    // Binary search for start index
    let startIndex = 0;
    let endIndex = itemPositions.length - 1;

    while (startIndex <= endIndex) {
      const middleIndex = Math.floor((startIndex + endIndex) / 2);
      const middleItem = itemPositions[middleIndex];

      if (middleItem.top + middleItem.height <= containerTop) {
        startIndex = middleIndex + 1;
      } else {
        endIndex = middleIndex - 1;
      }
    }

    const visibleStartIndex = Math.max(0, startIndex - overscan);

    // Find end index
    let visibleEndIndex = visibleStartIndex;
    while (
      visibleEndIndex < itemPositions.length &&
      itemPositions[visibleEndIndex].top < containerBottom + (overscan * estimatedItemHeight)
    ) {
      visibleEndIndex++;
    }

    visibleEndIndex = Math.min(itemPositions.length - 1, visibleEndIndex);

    endMeasure('virtual-scroll-calculate-range');
    return { startIndex: visibleStartIndex, endIndex: visibleEndIndex };
  }, [scrollTop, containerHeight, itemPositions, overscan, estimatedItemHeight, startMeasure, endMeasure]);

  // Get visible items
  const visibleItems = useMemo(() => {
    return items.slice(visibleRange.startIndex, visibleRange.endIndex + 1);
  }, [items, visibleRange]);

  // Handle scroll
  const handleScroll = useCallback((event: React.UIEvent<HTMLDivElement>) => {
    const newScrollTop = event.currentTarget.scrollTop;
    const newScrollLeft = event.currentTarget.scrollLeft;
    
    setScrollTop(newScrollTop);
    setIsScrolling(true);
    
    onScroll?.(newScrollTop, newScrollLeft);

    // Clear existing timeout
    if (scrollingTimeoutRef.current) {
      clearTimeout(scrollingTimeoutRef.current);
    }

    // Set new timeout to detect scroll end
    scrollingTimeoutRef.current = setTimeout(() => {
      setIsScrolling(false);
    }, 150);

    // Load more items if needed
    if (loadMoreItems && hasNextPage && !isLoading) {
      const { endIndex } = visibleRange;
      if (endIndex >= items.length - threshold) {
        loadMoreItems(items.length, items.length + 50);
      }
    }
  }, [onScroll, loadMoreItems, hasNextPage, isLoading, visibleRange, items.length, threshold]);

  // Handle items rendered
  useEffect(() => {
    onItemsRendered?.(visibleRange.startIndex, visibleRange.endIndex, visibleItems);
  }, [visibleRange.startIndex, visibleRange.endIndex, visibleItems, onItemsRendered]);

  // Scroll to index
  useEffect(() => {
    if (scrollToIndex !== undefined && containerRef.current && itemPositions[scrollToIndex]) {
      const item = itemPositions[scrollToIndex];
      let scrollTop: number;

      switch (scrollToAlignment) {
        case 'start':
          scrollTop = item.top;
          break;
        case 'center':
          scrollTop = item.top - (containerHeight - item.height) / 2;
          break;
        case 'end':
          scrollTop = item.top - containerHeight + item.height;
          break;
        case 'auto':
        default:
          const currentScrollTop = containerRef.current.scrollTop;
          const itemTop = item.top;
          const itemBottom = item.top + item.height;
          const containerTop = currentScrollTop;
          const containerBottom = currentScrollTop + containerHeight;

          if (itemTop < containerTop) {
            scrollTop = itemTop;
          } else if (itemBottom > containerBottom) {
            scrollTop = itemBottom - containerHeight;
          } else {
            return; // Item is already visible
          }
          break;
      }

      containerRef.current.scrollTop = Math.max(0, Math.min(scrollTop, totalHeight - containerHeight));
    }
  }, [scrollToIndex, scrollToAlignment, containerHeight, itemPositions, totalHeight]);

  // Measure item height after render
  const measureItemHeight = useCallback((index: number, element: HTMLElement) => {
    const height = element.getBoundingClientRect().height;
    const currentHeight = measuredHeightsRef.current.get(index);
    
    if (currentHeight !== height) {
      measuredHeightsRef.current.set(index, height);
      // Force re-calculation of positions on next render
      // This is handled by the useMemo dependency on measuredHeightsRef
    }
  }, []);

  // Render visible items
  const renderedItems = useMemo(() => {
    return visibleItems.map((item, relativeIndex) => {
      const absoluteIndex = visibleRange.startIndex + relativeIndex;
      const position = itemPositions[absoluteIndex];
      
      if (!position) return null;

      const style: React.CSSProperties = {
        position: 'absolute',
        top: position.top,
        left: 0,
        right: 0,
        height: position.height,
      };

      return (
        <div
          key={item.id}
          style={style}
          ref={(element) => {
            if (element) {
              measureItemHeight(absoluteIndex, element);
            }
          }}
        >
          {renderItem(item, absoluteIndex, style)}
        </div>
      );
    });
  }, [visibleItems, visibleRange.startIndex, itemPositions, renderItem, measureItemHeight]);

  return (
    <div
      ref={containerRef}
      className={`virtual-scroll-container ${className}`}
      style={{
        height: containerHeight,
        overflow: 'auto',
        position: 'relative',
      }}
      onScroll={handleScroll}
    >
      <div
        className="virtual-scroll-content"
        style={{
          height: totalHeight,
          position: 'relative',
        }}
      >
        {renderedItems}
        
        {/* Loading indicator */}
        {isLoading && (
          <div
            style={{
              position: 'absolute',
              top: totalHeight,
              left: 0,
              right: 0,
              height: 50,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
            }}
          >
            <div className="flex items-center space-x-2">
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
              <span className="text-sm text-gray-600">Loading more items...</span>
            </div>
          </div>
        )}
      </div>
      
      {/* Scroll indicator */}
      {isScrolling && (
        <div className="absolute top-2 right-2 bg-black bg-opacity-75 text-white px-2 py-1 rounded text-xs">
          {Math.round((scrollTop / (totalHeight - containerHeight)) * 100)}%
        </div>
      )}
    </div>
  );
}

// Hook for virtual scrolling
export const useVirtualScroll = <T extends VirtualScrollItem>(
  items: T[],
  containerHeight: number,
  itemHeight: number | ((item: T, index: number) => number)
) => {
  const [scrollTop, setScrollTop] = useState(0);
  const [visibleRange, setVisibleRange] = useState({ startIndex: 0, endIndex: 0 });

  const scrollToIndex = useCallback((index: number, alignment: 'start' | 'center' | 'end' | 'auto' = 'auto') => {
    // Implementation would depend on the container ref
    console.log(`Scroll to index ${index} with alignment ${alignment}`);
  }, []);

  const scrollToTop = useCallback(() => {
    setScrollTop(0);
  }, []);

  const scrollToBottom = useCallback(() => {
    // Implementation would depend on total height calculation
    console.log('Scroll to bottom');
  }, []);

  return {
    scrollTop,
    visibleRange,
    scrollToIndex,
    scrollToTop,
    scrollToBottom,
  };
};

// Grid virtual scrolling component
export interface VirtualGridProps<T extends VirtualScrollItem> {
  items: T[];
  itemWidth: number;
  itemHeight: number;
  containerWidth: number;
  containerHeight: number;
  renderItem: (item: T, index: number, style: React.CSSProperties) => React.ReactNode;
  overscan?: number;
  className?: string;
  gap?: number;
}

export function VirtualGrid<T extends VirtualScrollItem>({
  items,
  itemWidth,
  itemHeight,
  containerWidth,
  containerHeight,
  renderItem,
  overscan = 5,
  className = '',
  gap = 0,
}: VirtualGridProps<T>) {
  const [scrollTop, setScrollTop] = useState(0);
  const [scrollLeft, setScrollLeft] = useState(0);
  
  const columnsCount = Math.floor((containerWidth + gap) / (itemWidth + gap));
  const rowsCount = Math.ceil(items.length / columnsCount);
  
  const totalHeight = rowsCount * (itemHeight + gap) - gap;
  const totalWidth = columnsCount * (itemWidth + gap) - gap;

  const visibleRowStart = Math.max(0, Math.floor(scrollTop / (itemHeight + gap)) - overscan);
  const visibleRowEnd = Math.min(
    rowsCount - 1,
    Math.ceil((scrollTop + containerHeight) / (itemHeight + gap)) + overscan
  );

  const visibleColumnStart = Math.max(0, Math.floor(scrollLeft / (itemWidth + gap)) - overscan);
  const visibleColumnEnd = Math.min(
    columnsCount - 1,
    Math.ceil((scrollLeft + containerWidth) / (itemWidth + gap)) + overscan
  );

  const visibleItems = [];
  for (let row = visibleRowStart; row <= visibleRowEnd; row++) {
    for (let col = visibleColumnStart; col <= visibleColumnEnd; col++) {
      const index = row * columnsCount + col;
      if (index < items.length) {
        const item = items[index];
        const style: React.CSSProperties = {
          position: 'absolute',
          left: col * (itemWidth + gap),
          top: row * (itemHeight + gap),
          width: itemWidth,
          height: itemHeight,
        };
        
        visibleItems.push(
          <div key={item.id} style={style}>
            {renderItem(item, index, style)}
          </div>
        );
      }
    }
  }

  const handleScroll = (event: React.UIEvent<HTMLDivElement>) => {
    setScrollTop(event.currentTarget.scrollTop);
    setScrollLeft(event.currentTarget.scrollLeft);
  };

  return (
    <div
      className={`virtual-grid-container ${className}`}
      style={{
        width: containerWidth,
        height: containerHeight,
        overflow: 'auto',
        position: 'relative',
      }}
      onScroll={handleScroll}
    >
      <div
        className="virtual-grid-content"
        style={{
          width: totalWidth,
          height: totalHeight,
          position: 'relative',
        }}
      >
        {visibleItems}
      </div>
    </div>
  );
}

export default VirtualScrollList;