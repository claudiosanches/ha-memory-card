import { Stack, Tabs } from '@mantine/core';
import { getRouteApi } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { useCanManage, useStatus } from '../../api/queries';
import { PageHeader } from '../../components/PageHeader';
import { ToolTabList } from '../../components/ToolTabList';
import { DuplicatesTab } from '../DuplicatesPage';
import { SetupTab } from '../setup/SetupPage';
import { IgdbTab } from './IgdbTab';

export const MAINTENANCE_TABS = ['missing', 'duplicates', 'igdb'] as const;
export type MaintenanceTab = (typeof MAINTENANCE_TABS)[number];

const route = getRouteApi('/maintenance');

export function MaintenancePage() {
  const { t } = useTranslation();
  const canManage = useCanManage();
  const status = useStatus();
  const showIgdb = canManage && status.data?.igdb_configured;
  const visibleTabs = MAINTENANCE_TABS.filter((key) => {
    if (key === 'missing') return canManage;
    if (key === 'igdb') return showIgdb;
    return true;
  });

  const defaultTab = visibleTabs[0] ?? 'duplicates';
  const { tab } = route.useSearch();
  const activeTab = tab && visibleTabs.includes(tab) ? tab : defaultTab;
  const navigate = route.useNavigate();

  if (visibleTabs.length === 1) {
    return (
      <Stack gap="section">
        <PageHeader title={t('maintenance.tabs.duplicates')} description={t('maintenance.viewerIntro')} />
        <DuplicatesTab />
      </Stack>
    );
  }

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.maintenance')} description={t('maintenance.intro')} />
      <Tabs
        value={activeTab}
        onChange={(value) => void navigate({ to: '/maintenance', search: { tab: (value ?? defaultTab) as MaintenanceTab } })}
        keepMounted={false}
      >
        <ToolTabList
          label={t('nav.maintenance')}
          tabs={visibleTabs.map((key) => ({ value: key, label: t(`maintenance.tabs.${key}`) }))}
        />
        {canManage && (
          <Tabs.Panel value="missing">
            <SetupTab />
          </Tabs.Panel>
        )}
        <Tabs.Panel value="duplicates">
          <DuplicatesTab />
        </Tabs.Panel>
        {showIgdb && (
          <Tabs.Panel value="igdb">
            <IgdbTab />
          </Tabs.Panel>
        )}
      </Tabs>
    </Stack>
  );
}
