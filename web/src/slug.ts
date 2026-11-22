/** Matches the server: lowercase ASCII letters, digits and hyphens, starting and ending with a letter or digit. */
export const SLUG_MAX = 64;
const SLUG = /^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?$/;

export function isSlug(value: string): boolean {
  return value.length <= SLUG_MAX && SLUG.test(value);
}

/**
 * What someone types into a slug field, made valid as they type: accents
 * dropped, lowercase, other characters made one hyphen. A trailing hyphen
 * stays so the next word can follow it.
 */
export function sanitizeSlug(text: string): string {
  return text
    .normalize('NFKD')
    .replace(/\p{M}/gu, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+/, '')
    .slice(0, SLUG_MAX);
}

/** The slug for a name, as the server makes it. */
export function slugify(name: string): string {
  return trimSlug(sanitizeSlug(name));
}

export function trimSlug(slug: string): string {
  return slug.replace(/-+$/, '');
}
