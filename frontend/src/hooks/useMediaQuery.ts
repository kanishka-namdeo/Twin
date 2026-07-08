'use client';

import { useState, useEffect } from 'react';

/**
 * Hook to detect if viewport matches a media query.
 * Uses window.matchMedia for optimal performance (no resize event listeners).
 * 
 * @param query - CSS media query string (e.g., '(max-width: 899px)')
 * @returns boolean indicating if the media query matches
 */
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(false);

  useEffect(() => {
    const media = window.matchMedia(query);
    setMatches(media.matches);

    const listener = (event: MediaQueryListEvent) => {
      setMatches(event.matches);
    };

    media.addEventListener('change', listener);
    return () => media.removeEventListener('change', listener);
  }, [query]);

  return matches;
}

/**
 * Desktop viewport breakpoints for Tauri app.
 * Based on design system layout.md specifications.
 */
export const VIEWPORT_BREAKPOINTS = {
  /** Sidebar should collapse to icon-only mode */
  compact: '(max-width: 899px)',
  /** Standard two-panel layout */
  standard: '(min-width: 900px) and (max-width: 1199px)',
  /** Comfortable layout with more content visible */
  comfortable: '(min-width: 1200px) and (max-width: 1599px)',
  /** Extended layout for wide monitors */
  extended: '(min-width: 1600px)',
} as const;
