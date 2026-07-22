/**
 * Date Formatting Utilities for West Africa Time (Africa/Lagos, WAT, UTC+1)
 */
export const LAGOS_TIMEZONE = 'Africa/Lagos';

/**
 * Format a date string or timestamp into Lagos local date & time.
 * Example: "Jul 22, 2026, 11:45 PM"
 */
export function formatLagosDateTime(
  dateInput?: string | number | Date | null,
  options?: Intl.DateTimeFormatOptions
): string {
  if (!dateInput) return 'N/A';
  const date = new Date(dateInput);
  if (isNaN(date.getTime())) return 'N/A';

  return date.toLocaleString('en-US', {
    timeZone: LAGOS_TIMEZONE,
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
    hour12: true,
    ...options,
  });
}

/**
 * Format a date string or timestamp into Lagos local date only.
 * Example: "Jul 22, 2026"
 */
export function formatLagosDate(
  dateInput?: string | number | Date | null,
  options?: Intl.DateTimeFormatOptions
): string {
  if (!dateInput) return 'N/A';
  const date = new Date(dateInput);
  if (isNaN(date.getTime())) return 'N/A';

  return date.toLocaleDateString('en-US', {
    timeZone: LAGOS_TIMEZONE,
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    ...options,
  });
}

/**
 * Format a date string or timestamp into Lagos local time only.
 * Example: "11:45:12 PM"
 */
export function formatLagosTime(
  dateInput?: string | number | Date | null,
  options?: Intl.DateTimeFormatOptions
): string {
  if (!dateInput) return 'N/A';
  const date = new Date(dateInput);
  if (isNaN(date.getTime())) return 'N/A';

  return date.toLocaleTimeString('en-US', {
    timeZone: LAGOS_TIMEZONE,
    hour: 'numeric',
    minute: '2-digit',
    second: '2-digit',
    hour12: true,
    ...options,
  });
}
