import { Tabs } from '@mantine/core';
import type { CSSProperties } from 'react';
import classes from './ToolTabList.module.css';

type Props = {
  label: string;
  tabs: { value: string; label: string }[];
};

export function ToolTabList({ label, tabs }: Props) {
  return (
    <Tabs.List
      aria-label={label}
      mb="lg"
      className={classes.tabs}
      style={{ '--tab-columns': tabs.length > 3 ? 2 : tabs.length } as CSSProperties}
    >
      {tabs.map((tab) => (
        <Tabs.Tab key={tab.value} value={tab.value}>{tab.label}</Tabs.Tab>
      ))}
    </Tabs.List>
  );
}
