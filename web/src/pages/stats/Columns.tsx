import { Tooltip } from '@mantine/core';
import classes from './Columns.module.css';

export type Column = {
  key: string;
  /** Under the column; left blank to thin out crowded labels. */
  short: string;
  /** The period in full, for the tooltip and screen readers. */
  long: string;
  value: number;
  figure: string;
};

/** One column per period, oldest first, with the peak's value printed above it. */
export function Columns({ columns, label }: { columns: Column[]; label: string }) {
  const max = Math.max(1, ...columns.map((c) => c.value));
  const peak = columns.reduce<Column | undefined>((best, c) => (c.value > (best?.value ?? 0) ? c : best), undefined);
  // Keep labels readable when there are many columns, counting back from the latest.
  const every = Math.ceil(columns.length / 12);

  return (
    <div
      className={classes.columns}
      style={{ gridTemplateColumns: `repeat(${columns.length}, minmax(0, 1fr))` }}
      role="list"
      aria-label={label}
    >
      {columns.map((c, index) => {
        const text = `${c.long}: ${c.figure}`;
        return (
          <Tooltip key={c.key} label={text} withArrow>
            <div className={classes.column} role="listitem" aria-label={text} tabIndex={0}>
              <span className={classes.value}>{c === peak ? c.value : ''}</span>
              <span className={classes.plot}>
                <span className={classes.bar} style={{ height: `${(c.value / max) * 100}%` }} />
              </span>
              <span className={classes.label}>{(columns.length - 1 - index) % every === 0 ? c.short : ''}</span>
            </div>
          </Tooltip>
        );
      })}
    </div>
  );
}
