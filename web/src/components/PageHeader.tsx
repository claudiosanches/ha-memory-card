import { Group, Text, Title } from '@mantine/core';
import type { ReactNode } from 'react';
import { Description } from './Description';
import classes from './PageHeader.module.css';

type Props = {
  title: string;
  description: string;
  count?: string;
  countTestId?: string;
  children?: ReactNode;
};

export function PageHeader({ title, description, count, countTestId, children }: Props) {
  return (
    <Group component="header" justify="space-between" align="center" className={classes.heading}>
      <div className={classes.intro}>
        <Group gap="sm">
          <Title order={1} className={classes.title}>{title}</Title>
          {count && <Text className={classes.count} data-testid={countTestId} aria-live="polite">{count}</Text>}
        </Group>
        <Description>{description}</Description>
      </div>
      {children}
    </Group>
  );
}
