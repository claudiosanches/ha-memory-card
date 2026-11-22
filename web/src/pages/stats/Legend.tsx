import { Link } from '@tanstack/react-router';
import type { LibrarySearch } from '../library/search';
import classes from './charts.module.css';

export type LegendRow = {
  key: string;
  label: string;
  color: string;
  figure: string;
  /** Library filters that list what this row counts. */
  search?: LibrarySearch;
};

/** Names, colors and figures for a chart's parts, each linking to its games when it can. */
export function Legend({
  rows,
  active,
  onActive,
}: {
  rows: LegendRow[];
  active: string | null;
  onActive: (key: string | null) => void;
}) {
  return (
    <ul className={classes.legend}>
      {rows.map((row) => (
        <li
          key={row.key}
          className={classes.row}
          data-active={row.key === active ? '' : undefined}
          onMouseEnter={() => onActive(row.key)}
          onMouseLeave={() => onActive(null)}
        >
          <span className={classes.swatch} style={{ background: row.color }} aria-hidden />
          <span className={classes.label}>
            {row.search ? (
              <Link to="/" search={row.search} className={classes.link}>
                {row.label}
              </Link>
            ) : (
              row.label
            )}
          </span>
          <span className={classes.figure}>{row.figure}</span>
        </li>
      ))}
    </ul>
  );
}

/** A part's share of a whole, with "<1%" for slivers. */
export function shareFormat(locale: string, total: number) {
  const percent = new Intl.NumberFormat(locale, { style: 'percent', maximumFractionDigits: 0 });
  return (value: number) => (value / total < 0.005 ? `<${percent.format(0.01)}` : percent.format(value / total));
}
