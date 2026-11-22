import { Stack, Tabs } from '@mantine/core';
import { getRouteApi } from '@tanstack/react-router';
import { PageHeader } from '../../components/PageHeader';
import { ToolTabList } from '../../components/ToolTabList';
import { useTranslation } from 'react-i18next';
import { DevicesTab } from './DevicesTab';
import { PlatformsTab } from './PlatformsTab';
import { PlayStatusesTab } from './PlayStatusesTab';
import { StoresTab } from './StoresTab';

export const MANAGE_TABS = ['devices', 'stores', 'platforms', 'playStatuses'] as const;
export type ManageTab = (typeof MANAGE_TABS)[number];

const route = getRouteApi('/manage');

export function ManagePage() {
  const { t } = useTranslation();
  const { tab = 'devices' } = route.useSearch();
  const navigate = route.useNavigate();

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.manage')} description={t('manage.intro')} />
      <Tabs
        value={tab}
        onChange={(value) => void navigate({ to: '/manage', search: { tab: (value ?? 'devices') as ManageTab } })}
        keepMounted={false}
      >
        <ToolTabList label={t('nav.manage')} tabs={MANAGE_TABS.map((key) => ({ value: key, label: t(`manage.tabs.${key}`) }))} />
        <Tabs.Panel value="devices">
          <DevicesTab />
        </Tabs.Panel>
        <Tabs.Panel value="stores">
          <StoresTab />
        </Tabs.Panel>
        <Tabs.Panel value="platforms">
          <PlatformsTab />
        </Tabs.Panel>
        <Tabs.Panel value="playStatuses">
          <PlayStatusesTab />
        </Tabs.Panel>
      </Tabs>
    </Stack>
  );
}
