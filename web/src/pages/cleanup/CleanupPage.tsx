import { Stack, Tabs } from '@mantine/core';
import { getRouteApi } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';
import { useCanManage, useStatus } from '../../api/queries';
import { PageHeader } from '../../components/PageHeader';
import { ToolTabList } from '../../components/ToolTabList';
import { DuplicatesTab } from '../DuplicatesPage';
import { SetupTab } from '../setup/SetupPage';
import { IgdbTab } from './IgdbTab';

export const CLEANUP_TABS = ['missing', 'duplicates', 'igdb'] as const;
export type CleanupTab = (typeof CLEANUP_TABS)[number];

const route = getRouteApi('/cleanup');

export function CleanupPage() {
  const { t } = useTranslation();
  const canManage = useCanManage();
  const status = useStatus();
  const showIgdb = canManage && status.data?.igdb_configured;
  const visibleTabs = CLEANUP_TABS.filter((key) => {
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
        <PageHeader title={t('nav.cleanup')} description={t('cleanup.viewerIntro')} />
        <DuplicatesTab showIntro={false} />
      </Stack>
    );
  }

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.cleanup')} description={t('cleanup.intro')} />
      <Tabs
        value={activeTab}
        onChange={(value) => void navigate({ to: '/cleanup', search: { tab: (value ?? defaultTab) as CleanupTab } })}
        keepMounted={false}
      >
        <ToolTabList
          label={t('nav.cleanup')}
          tabs={visibleTabs.map((key) => ({ value: key, label: t(`cleanup.tabs.${key}`) }))}
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
