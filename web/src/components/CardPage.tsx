import { Stack } from '@mantine/core';
import type { ReactNode } from 'react';
import { PageHeader } from './PageHeader';

export function CardPage({ title, description, children }: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <Stack gap="section">
      <PageHeader title={title} description={description} />
      <Stack gap="card">{children}</Stack>
    </Stack>
  );
}
