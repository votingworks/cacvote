const countFormatter = new Intl.NumberFormat(undefined, { useGrouping: true });

/**
 * Format integers for display as whole numbers, i.e. a count of something.
 */
export function count(value: number): string {
  return countFormatter.format(value);
}

function interpolate(
  template: string,
  values: Record<string, string | number>
): string {
  return template.replace(/{{\s*([a-zA-Z0-9_]+)\s*}}/g, (_match, key) =>
    Object.hasOwn(values, key) ? `${values[key]}` : ''
  );
}

/**
 * Format a number as a count of something, with a phrase that depends on the
 * count.
 *
 * @example
 *
 *  countPhrase(0, { one: '1 item', many: '{{count}} items' }) // '0 items'
 */
export function countPhrase(
  value: number,
  {
    zero,
    one,
    many,
  }: {
    zero?: string;
    one: string;
    many: string;
  }
): string {
  let template = '';
  if (value === 0 && zero) {
    template = zero;
  } else if (value === 1) {
    template = one;
  } else {
    template = many;
  }
  return interpolate(template, { count: countFormatter.format(value) });
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

export const DEFAULT_LOCALE = 'en-US';

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

export function localeLongDate(time?: number | Date): string {
  return new Intl.DateTimeFormat(DEFAULT_LOCALE, {
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
