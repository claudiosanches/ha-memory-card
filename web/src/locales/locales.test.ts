import { describe, expect, it } from 'vitest';
import en from './en.json';

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
