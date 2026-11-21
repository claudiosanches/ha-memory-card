import { describe, expect, it } from 'vitest';
import en from './en.json';
import ptBR from './pt-BR.json';

function leaves(node: unknown, path = ''): [string, unknown][] {
  if (node && typeof node === 'object') {
    return Object.entries(node).flatMap(([key, value]) => leaves(value, path ? `${path}.${key}` : key));
  }
  return [[path, node]];
}

describe('en.json', () => {
  it('has a non-empty string for every key', () => {
    for (const [key, value] of leaves(en)) {
      expect(typeof value, key).toBe('string');
      expect((value as string).trim(), key).not.toBe('');
    }
  });
});

describe('pt-BR.json', () => {
  it('has every key of en.json, and only those, with the same placeholders', () => {
    const english = new Map(leaves(en));
    const portuguese = new Map(leaves(ptBR));
    expect([...portuguese.keys()].sort()).toEqual([...english.keys()].sort());
    const placeholders = (text: unknown) => [...String(text).matchAll(/\{\{(\w+)\}\}/g)].map((m) => m[1]).sort();
    for (const [key, value] of portuguese) {
      expect(typeof value, key).toBe('string');
      expect((value as string).trim(), key).not.toBe('');
      expect(placeholders(value), key).toEqual(placeholders(english.get(key)));
    }
  });
});
