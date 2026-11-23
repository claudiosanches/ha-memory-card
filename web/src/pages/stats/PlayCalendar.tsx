import { Text, Tooltip } from '@mantine/core';
import type { TFunction } from 'i18next';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import type { Stats } from '../../api/types';
import classes from './PlayCalendar.module.css';

const WEEKS = 53;

type Day = { date: Date; iso: string; sessions: number; minutes: number };
/** The day under the pointer and its cell, which the tooltip points at. */
type Active = { day: Day; cell: HTMLElement };

function iso(date: Date): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}

/** The weekday a locale starts its weeks on, 0 for Sunday; Monday when the browser cannot say. */
export function firstWeekday(locale: string): number {
  try {
    const info = new Intl.Locale(locale) as Intl.Locale & {
      getWeekInfo?: () => { firstDay: number };
      weekInfo?: { firstDay: number };
    };
    const firstDay = (info.getWeekInfo?.() ?? info.weekInfo)?.firstDay;
    return firstDay ? firstDay % 7 : 1;
  } catch {
    return 1;
  }
}

/** Darker the more sessions a day had. */
function level(sessions: number): number {
  return Math.min(sessions, 4);
}

/** Weeks as columns, oldest first, ending with the week of `today`. */
export function calendarWeeks(today: Date, startDay: number, played: Map<string, Day>): (Day | null)[][] {
  const end = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  const start = new Date(end);
  start.setDate(start.getDate() - (WEEKS - 1) * 7 - ((end.getDay() - startDay + 7) % 7));
  const weeks: (Day | null)[][] = [];
  for (let w = 0; w < WEEKS; w++) {
    const week: (Day | null)[] = [];
    for (let d = 0; d < 7; d++) {
      const date = new Date(start.getFullYear(), start.getMonth(), start.getDate() + w * 7 + d);
      const key = iso(date);
      week.push(date > end ? null : (played.get(key) ?? { date, iso: key, sessions: 0, minutes: 0 }));
    }
    weeks.push(week);
  }
  return weeks;
}

export function PlayCalendar({
  stats,
  locale,
  playTime,
}: {
  stats: Stats;
  locale: string;
  playTime: (minutes: number, t: TFunction) => string;
}) {
  const { t } = useTranslation();
  const scroller = useRef<HTMLDivElement>(null);
  const [active, setActive] = useState<Active | null>(null);

  const played = new Map(
    stats.play_days.map((d) => {
      const [y, m, day] = d.day.split('-').map(Number);
      return [d.day, { date: new Date(y!, m! - 1, day!), iso: d.day, sessions: d.sessions, minutes: d.minutes }];
    }),
  );
  const startDay = firstWeekday(locale);
  const weeks = calendarWeeks(new Date(), startDay, played);
  const shownDays = weeks.flat().filter((d): d is Day => d !== null && d.sessions > 0);
  const sessions = shownDays.reduce((sum, d) => sum + d.sessions, 0);
  const summary = t('stats.playedDays', {
    sessions: t('stats.sessions', { count: sessions }),
    days: t('stats.daysCount', { count: shownDays.length }),
  });

  // Recent weeks matter most, so a narrow screen starts scrolled to them.
  useEffect(() => {
    const el = scroller.current;
    if (el) el.scrollLeft = el.scrollWidth;
  }, []);

  const month = (date: Date) => date.toLocaleDateString(locale, { month: 'short' });
  const weekday = (offset: number) =>
    new Date(2024, 0, 7 + ((startDay + offset) % 7)).toLocaleDateString(locale, { weekday: 'short' });
  const describe = (day: Day) =>
    [
      day.date.toLocaleDateString(locale, { weekday: 'short', day: 'numeric', month: 'short', year: 'numeric' }),
      day.sessions ? t('stats.sessions', { count: day.sessions }) : t('stats.noSessionsDay'),
      day.minutes > 0 ? playTime(day.minutes, t) : null,
    ]
      .filter(Boolean)
      .join(t('common.separator'));

  // A month name over the first week of each month. The first column's month
  // goes unnamed when the next one starts too close for both to fit.
  const months = weeks.map((week, index) => {
    const first = week[0]!.date;
    const starts = index === 0 || weeks[index - 1]![0]!.date.getMonth() !== first.getMonth();
    return starts ? month(first) : '';
  });
  if (months.slice(1, 3).some(Boolean)) months[0] = '';

  return (
    <div className={classes.calendar}>
      <Text size="sm" c="dimmed">{t('stats.lastYear')}{t('common.separator')}{summary}</Text>
      <div className={classes.scroller} ref={scroller}>
        <div className={classes.grid} role="img" aria-label={summary} onMouseLeave={() => setActive(null)}>
          <span />
          {months.map((label, index) => (
            <span key={index} className={classes.month}>{label}</span>
          ))}
          {Array.from({ length: 7 }, (_, row) => (
            <Row key={row} row={row} label={row % 2 === 1 ? weekday(row) : ''} weeks={weeks} onActive={setActive} />
          ))}
        </div>
        {/* Over the day rather than beside the pointer, so it flips and shifts to stay in the window. */}
        {active && <Tooltip label={describe(active.day)} target={active.cell} opened withArrow />}
      </div>
      <div className={classes.scale} aria-hidden>
        {t('stats.less')}
        {[0, 1, 2, 3, 4].map((n) => (
          <span key={n} className={classes.cell} data-level={n} />
        ))}
        {t('stats.more')}
      </div>
    </div>
  );
}

function Row({
  row,
  label,
  weeks,
  onActive,
}: {
  row: number;
  label: string;
  weeks: (Day | null)[][];
  onActive: (active: Active | null) => void;
}) {
  return (
    <>
      <span className={classes.weekday}>{label}</span>
      {weeks.map((week, w) => {
        const day = week[row];
        return day ? (
          <span
            key={w}
            className={classes.cell}
            data-level={level(day.sessions)}
            onMouseEnter={(event) => onActive({ day, cell: event.currentTarget })}
          />
        ) : (
          <span key={w} className={classes.cell} data-future="" onMouseEnter={() => onActive(null)} />
        );
      })}
    </>
  );
}
