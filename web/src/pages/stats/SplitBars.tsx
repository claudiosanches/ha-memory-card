import { Link } from '@tanstack/react-router';
import type { LibrarySearch } from '../library/search';
import charts from './charts.module.css';
import classes from './SplitBars.module.css';

export type SplitPart = { key: string; label: string; color: string };
export type SplitRow = { key: string; label: string; search?: LibrarySearch; values: number[]; figure: string };

/** A bar per row, as long as its total against the largest, split into the same parts in the same order. */
export function SplitBars({ parts, rows }: { parts: SplitPart[]; rows: SplitRow[] }) {
  const max = Math.max(1, ...rows.map((row) => row.values.reduce((a, b) => a + b, 0)));
  return (
    <>
      <ul className={charts.keys}>
        {parts.map((part) => (
          <li key={part.key}>
            <span className={charts.swatch} style={{ background: part.color }} aria-hidden />
            {part.label}
          </li>
        ))}
      </ul>
      <ul className={classes.rows}>
        {rows.map((row) => (
          <li key={row.key} className={classes.row}>
            <span className={classes.label}>
              {row.search ? (
                <Link to="/" search={row.search} className={charts.link}>
                  {row.label}
                </Link>
              ) : (
                row.label
              )}
            </span>
            <span className={classes.track} aria-hidden>
              {row.values.map((value, i) =>
                value > 0 ? (
                  <span
                    key={parts[i]!.key}
                    className={classes.segment}
                    style={{ width: `${(value / max) * 100}%`, background: parts[i]!.color }}
                  />
                ) : null,
              )}
            </span>
            <span className={charts.figure}>{row.figure}</span>
          </li>
        ))}
      </ul>
    </>
  );
}
