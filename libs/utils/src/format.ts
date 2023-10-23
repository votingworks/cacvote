import { assertDefined } from '@votingworks/basics';
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
  return interpolate(template, {
    count: new Intl.NumberFormat(locale).format(value),
  });
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

export function languageDisplayName(params: {
  languageCode: LanguageCode;

  /** @default {@link params.languageCode} */
  displayLanguageCode?: LanguageCode;

  /** @default 'narrow' */
  style?: Intl.RelativeTimeFormatStyle;
}): string {
  const {
    languageCode,
    displayLanguageCode = languageCode,
    style = 'narrow',
  } = params;

  return assertDefined(
    // TODO(kofi): This util doesn't currently have a comprehensive list of
    // native language names for all languages, so probably better to find a
    // reliably sourced list and hardcode the mappings instead.
    // This at least works for the top 10 spoken in the US and is sufficient for
    // cert.
    new Intl.DisplayNames([displayLanguageCode], {
      style,
      type: 'language',
      fallback: 'none',
    }).of(languageCode),
    `unexpected missing language display name for ${languageCode} in ${displayLanguageCode}`
  );
}
