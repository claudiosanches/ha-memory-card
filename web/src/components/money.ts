/** Usual currency for a locale's region, used before any copy has a price. */
const REGION_CURRENCY: Record<string, string> = {
  US: 'USD', GB: 'GBP', BR: 'BRL', JP: 'JPY', CA: 'CAD', AU: 'AUD', NZ: 'NZD', CH: 'CHF', CN: 'CNY',
  KR: 'KRW', MX: 'MXN', IN: 'INR', SE: 'SEK', NO: 'NOK', DK: 'DKK', PL: 'PLN', CZ: 'CZK', HU: 'HUF',
  AR: 'ARS', CL: 'CLP', CO: 'COP', ZA: 'ZAR', TR: 'TRY', HK: 'HKD', TW: 'TWD', SG: 'SGD',
  AT: 'EUR', BE: 'EUR', DE: 'EUR', ES: 'EUR', FI: 'EUR', FR: 'EUR', GR: 'EUR', IE: 'EUR', IT: 'EUR',
  NL: 'EUR', PT: 'EUR', SK: 'EUR', SI: 'EUR', LU: 'EUR', EE: 'EUR', LV: 'EUR', LT: 'EUR', HR: 'EUR',
};

export function localeCurrency(locale: string): string {
  try {
    const region = new Intl.Locale(locale).maximize().region;
    return (region && REGION_CURRENCY[region]) || 'USD';
  } catch {
    return 'USD';
  }
}

/** Every ISO 4217 currency the browser knows, labelled in the UI language. */
export function currencyOptions(locale: string): { value: string; label: string }[] {
  const names = new Intl.DisplayNames([locale], { type: 'currency' });
  return Intl.supportedValuesOf('currency').map((code) => ({ value: code, label: `${code} · ${names.of(code) ?? code}` }));
}

/** A decimal amount from the API, such as `59.99`, in the locale's currency format. */
export function formatMoney(amount: string, currency: string, locale: string): string {
  try {
    return new Intl.NumberFormat(locale, { style: 'currency', currency }).format(Number(amount));
  } catch {
    return `${amount} ${currency}`;
  }
}
