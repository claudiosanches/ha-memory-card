import { Stack, Text, Title } from '@mantine/core';
import type { ComponentType, ReactNode } from 'react';
import classes from './EmptyState.module.css';

type IconProps = { size?: number; stroke?: number; 'aria-hidden'?: boolean };

export function EmptyState({
  icon: Icon,
  title,
  description,
  action,
  compact = false,
}: {
  icon?: ComponentType<IconProps>;
  title?: string;
  description?: ReactNode;
  action?: ReactNode;
  compact?: boolean;
}) {
  return (
    <Stack
      align="center"
      gap={compact ? 'xs' : 'sm'}
      className={classes.root}
      data-compact={compact ? '' : undefined}
    >
      {Icon && <Icon size={compact ? 32 : 48} stroke={1.2} aria-hidden />}
      {title && (
        <Title order={compact ? 4 : 3} ta="center">
          {title}
        </Title>
      )}
      {description && (
        <Text c="dimmed" size={compact ? 'sm' : undefined} ta="center" className={classes.description}>
          {description}
        </Text>
      )}
      {action && <div className={classes.action}>{action}</div>}
    </Stack>
  );
}
