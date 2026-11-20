import { Stack, Text, Title } from '@mantine/core';
import { useTranslation } from 'react-i18next';

type Props = { titleKey: 'nav.library' | 'nav.duplicates' | 'nav.import' | 'nav.manage' };

export function PlaceholderPage({ titleKey }: Props) {
  const { t } = useTranslation();
  return (
    <Stack>
      <Title order={2}>{t(titleKey)}</Title>
      <Text c="dimmed">{t('placeholder.notBuilt')}</Text>
    </Stack>
  );
}
