import { Text } from '@mantine/core';
import type { ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import { useCanManage } from '../api/queries';

/** Shows its children to managers only, for pages a viewer reaches by URL. */
export function ManagerOnly({ children }: { children: ReactNode }) {
  const { t } = useTranslation();
  return useCanManage() ? children : <Text c="dimmed">{t('app.managersOnly')}</Text>;
}
