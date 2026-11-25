import { describe, expect, it } from 'vitest';
import { withColumns } from './importColumns';

describe('withColumns', () => {
  it('renames the headers and keeps the rows', () => {
    const text = '﻿Name,"Sys, tem",Stars\r\nZelda,Wii,5\r\n';
    expect(withColumns(text, ['game_title', 'platform', null], ',')).toBe(
      'game_title,platform,ignored 3\r\nZelda,Wii,5\r\n',
    );
    expect(withColumns('A;B\nx;y', ['notes', null], ';')).toBe('notes;ignored 2\nx;y');
  });

  it('replaces the header after blank lines, as the server reads it', () => {
    expect(withColumns('\r\n\nName\nGame\n', ['game_title'], ',')).toBe('\r\n\ngame_title\nGame\n');
  });
});
