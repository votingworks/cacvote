import { LanguageCode } from '@votingworks/types';

export const DEFAULT_LOCALE: LanguageCode = LanguageCode.ENGLISH;

/**
 * Format integers for display as whole numbers, i.e. a count of something.
 */
export function count(
  value: number,
  locale: LanguageCode = DEFAULT_LOCALE
): string {
  return new Intl.NumberFormat(locale, { useGrouping: true }).format(value);
}

/**
 * Format a number as a count of something, with a phrase that depends on the
 * count.
 *
 * @example
 *
 *  countPhrase(0, { one: '1 item', many: '{{count}} items' }) // '0 items'
 */
export function countPhrase({
  value,
  locale = DEFAULT_LOCALE,
  zero,
  one,
  many,
}: {
  value: number;
  locale?: LanguageCode;
  zero?: string;
  one: string;
  many: string;
}): string {
  let template = '';
  if (value === 0 && zero) {
    template = zero;
  } else if (value === 1) {
    template = one;
  } else {
    template = many;
  }
  return template.replaceAll(
    '{{count}}',
    new Intl.NumberFormat(locale).format(value)
  );
}

/**
 * Formats a number as a percentage.
 *
 * @example
 *   percent(0.591)                               // '59%'
 *   percent(0.591, { maximumFractionDigits: 1 }) // '59.1%'
 */
export function percent(
  value: number,
  { maximumFractionDigits = 0 } = {}
): string {
  const percentFormatter = new Intl.NumberFormat(undefined, {
    useGrouping: true,
    style: 'percent',
    maximumFractionDigits,
  });
  return percentFormatter.format(value);
}

export function localeLongDateAndTime(time?: number | Date): string {
  return new Intl.DateTimeFormat(DEFAULT_LOCALE, {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: 'numeric',
    hour: 'numeric',
    minute: 'numeric',
    second: 'numeric',
    timeZoneName: 'short',
  }).format(time);
}

export function localeWeekdayAndDate(time?: number | Date): string {
  return new Intl.DateTimeFormat(DEFAULT_LOCALE, {
    weekday: 'long',
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  }).format(time);
}

export function localeLongDate(
  time?: number | Date,
  locale: LanguageCode = DEFAULT_LOCALE
): string {
  return new Intl.DateTimeFormat(locale, {
    month: 'long',
    day: 'numeric',
    year: 'numeric',
  }).format(time);
}

export function localeDate(time?: number | Date): string {
  return new Intl.DateTimeFormat(DEFAULT_LOCALE, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(time);
}
