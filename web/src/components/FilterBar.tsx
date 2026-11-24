import type { ReactNode } from 'react';
import classes from './FilterBar.module.css';

export function FilterBar({ label, children }: { label: string; children: ReactNode }) {
  return <div role="group" aria-label={label} className={classes.filters}>{children}</div>;
}

type FilterButtonProps = {
  children: ReactNode;
  selected: boolean;
  onClick: () => void;
  count?: number;
};

export function FilterButton({ children, selected, onClick, count }: FilterButtonProps) {
  return (
    <button type="button" className={`mc-filter ${classes.filter}`} aria-pressed={selected} onClick={onClick}>
      {children}
      {count !== undefined && <span className={classes.filterCount}>{count}</span>}
    </button>
  );
}
