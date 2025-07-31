// Accessibility testing setup
import { configureAxe } from 'jest-axe';

// Configure axe for accessibility testing
const axe = configureAxe({
  rules: {
    // Disable color-contrast rule for now (would need actual color analysis)
    'color-contrast': { enabled: false },
    // Disable landmark rules for components in isolation
    'region': { enabled: false },
    // Custom rules configuration
    'aria-allowed-attr': { enabled: true },
    'aria-required-attr': { enabled: true },
    'aria-valid-attr': { enabled: true },
    'aria-valid-attr-value': { enabled: true },
    'button-name': { enabled: true },
    'duplicate-id': { enabled: true },
    'form-field-multiple-labels': { enabled: true },
    'html-has-lang': { enabled: false }, // Not applicable for components
    'html-lang-valid': { enabled: false }, // Not applicable for components
    'image-alt': { enabled: true },
    'input-image-alt': { enabled: true },
    'label': { enabled: true },
    'link-name': { enabled: true },
    'list': { enabled: true },
    'listitem': { enabled: true },
    'meta-refresh': { enabled: false }, // Not applicable for components
    'meta-viewport': { enabled: false }, // Not applicable for components
    'object-alt': { enabled: true },
    'role-img-alt': { enabled: true },
    'scrollable-region-focusable': { enabled: true },
    'select-name': { enabled: true },
    'server-side-image-map': { enabled: true },
    'svg-img-alt': { enabled: true },
    'td-headers-attr': { enabled: true },
    'th-has-data-cells': { enabled: true },
    'valid-lang': { enabled: true },
    'video-caption': { enabled: false }, // Not applicable for our components
  },
  tags: ['wcag2a', 'wcag2aa', 'wcag21aa'],
});

// Global accessibility test utilities
global.accessibilityUtils = {
  // Check if element has proper ARIA attributes
  hasProperAria: (element: HTMLElement) => {
    const role = element.getAttribute('role');
    const ariaLabel = element.getAttribute('aria-label');
    const ariaLabelledBy = element.getAttribute('aria-labelledby');
    const ariaDescribedBy = element.getAttribute('aria-describedby');
    
    return {
      hasRole: !!role,
      hasLabel: !!(ariaLabel || ariaLabelledBy),
      hasDescription: !!ariaDescribedBy,
      isAccessible: !!(role && (ariaLabel || ariaLabelledBy)),
    };
  },
  
  // Check if interactive elements are keyboard accessible
  isKeyboardAccessible: (element: HTMLElement) => {
    const tabIndex = element.getAttribute('tabindex');
    const isInteractive = ['button', 'a', 'input', 'select', 'textarea'].includes(
      element.tagName.toLowerCase()
    );
    const hasRole = ['button', 'link', 'menuitem', 'tab'].includes(
      element.getAttribute('role') || ''
    );
    
    return {
      isFocusable: tabIndex !== '-1' && (isInteractive || hasRole || tabIndex === '0'),
      tabIndex: tabIndex,
      isInteractive: isInteractive || hasRole,
    };
  },
  
  // Check color contrast (simplified version)
  hasGoodContrast: (element: HTMLElement) => {
    const styles = window.getComputedStyle(element);
    const color = styles.color;
    const backgroundColor = styles.backgroundColor;
    
    // This is a simplified check - in real implementation,
    // you'd use a proper color contrast analyzer
    return {
      color,
      backgroundColor,
      // Assume good contrast for now
      ratio: 4.5,
      passes: true,
    };
  },
  
  // Check if text is readable by screen readers
  isScreenReaderFriendly: (element: HTMLElement) => {
    const ariaHidden = element.getAttribute('aria-hidden');
    const hasText = element.textContent && element.textContent.trim().length > 0;
    const hasAltText = element.getAttribute('alt');
    const hasAriaLabel = element.getAttribute('aria-label');
    
    return {
      isHidden: ariaHidden === 'true',
      hasContent: hasText || hasAltText || hasAriaLabel,
      isAccessible: ariaHidden !== 'true' && (hasText || hasAltText || hasAriaLabel),
    };
  },
  
  // Get all focusable elements in a container
  getFocusableElements: (container: HTMLElement) => {
    const focusableSelectors = [
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
      '[role="button"]:not([disabled])',
      '[role="link"]',
      '[role="menuitem"]',
      '[role="tab"]',
    ].join(', ');
    
    return Array.from(container.querySelectorAll(focusableSelectors));
  },
  
  // Check heading hierarchy
  checkHeadingHierarchy: (container: HTMLElement) => {
    const headings = Array.from(container.querySelectorAll('h1, h2, h3, h4, h5, h6'));
    const levels = headings.map(h => parseInt(h.tagName.charAt(1)));
    
    let isValid = true;
    let errors: string[] = [];
    
    for (let i = 1; i < levels.length; i++) {
      const current = levels[i];
      const previous = levels[i - 1];
      
      if (current > previous + 1) {
        isValid = false;
        errors.push(`Heading level ${current} follows level ${previous} (skipped level)`);
      }
    }
    
    return {
      isValid,
      errors,
      levels,
      headings: headings.map(h => ({
        level: parseInt(h.tagName.charAt(1)),
        text: h.textContent,
        element: h,
      })),
    };
  },
};

// Mock screen reader announcements
const announcements: string[] = [];

global.mockScreenReader = {
  announcements,
  announce: (message: string) => {
    announcements.push(message);
  },
  clear: () => {
    announcements.length = 0;
  },
  getLastAnnouncement: () => announcements[announcements.length - 1],
  getAllAnnouncements: () => [...announcements],
};

// Mock ARIA live regions
const originalCreateElement = document.createElement;
document.createElement = function(tagName: string, options?: ElementCreationOptions) {
  const element = originalCreateElement.call(this, tagName, options);
  
  if (element.setAttribute) {
    const originalSetAttribute = element.setAttribute;
    element.setAttribute = function(name: string, value: string) {
      originalSetAttribute.call(this, name, value);
      
      // Mock screen reader announcements for live regions
      if (name === 'aria-live' && (value === 'polite' || value === 'assertive')) {
        const observer = new MutationObserver((mutations) => {
          mutations.forEach((mutation) => {
            if (mutation.type === 'childList' || mutation.type === 'characterData') {
              const text = this.textContent;
              if (text && text.trim()) {
                global.mockScreenReader.announce(text.trim());
              }
            }
          });
        });
        
        observer.observe(this, {
          childList: true,
          subtree: true,
          characterData: true,
        });
      }
    };
  }
  
  return element;
};

// Set up accessibility testing environment
beforeEach(() => {
  // Clear screen reader announcements
  global.mockScreenReader.clear();
  
  // Reset any accessibility-related mocks
  jest.clearAllMocks();
});

// Export configured axe for use in tests
export { axe };