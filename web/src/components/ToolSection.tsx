import { Card, Group, Stack, Title } from '@mantine/core';
import type { ReactNode } from 'react';
import { Description } from './Description';

type Props = {
  title: string;
  description?: ReactNode;
  action?: ReactNode;
  children?: ReactNode;
};

export function ToolSection({ title, description, action, children }: Props) {
  return (
    <Card component="section" withBorder radius="md">
      <Stack gap="md">
        <Group justify="space-between">
          <Stack gap={4}>
            <Title order={2} size="h4">{title}</Title>
            {description && <Description>{description}</Description>}
          </Stack>
          {action}
        </Group>
        {children}
      </Stack>
    </Card>
  );
}
