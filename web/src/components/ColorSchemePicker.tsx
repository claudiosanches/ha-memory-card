import { ActionIcon, Menu, SegmentedControl, useMantineColorScheme } from '@mantine/core';
import { IconCheck, IconDeviceDesktop, IconMoon, IconSun } from '@tabler/icons-react';
import { useTranslation } from 'react-i18next';

const choices = [
  { value: 'auto', icon: IconDeviceDesktop },
  { value: 'light', icon: IconSun },
  { value: 'dark', icon: IconMoon },
] as const;

export function ColorSchemePicker({ compact = false }: { compact?: boolean }) {
  const { t } = useTranslation();
  const { colorScheme, setColorScheme } = useMantineColorScheme();
  const SelectedIcon = choices.find((choice) => choice.value === colorScheme)?.icon ?? IconDeviceDesktop;

  if (!compact) {
    return (
      <SegmentedControl
        aria-label={t('settings.colorScheme')}
        value={colorScheme}
        onChange={(value) => {
          if (value === 'auto' || value === 'light' || value === 'dark') setColorScheme(value);
        }}
        data={choices.map(({ value }) => ({ value, label: t(`settings.schemes.${value}`) }))}
      />
    );
  }

  return (
    <Menu position="bottom-end" width={180}>
      <Menu.Target>
        <ActionIcon variant="default" size="lg" aria-label={t('settings.colorScheme')} title={t('settings.colorScheme')}>
          <SelectedIcon size={19} stroke={1.7} />
        </ActionIcon>
      </Menu.Target>
      <Menu.Dropdown>
        <Menu.Label>{t('settings.colorScheme')}</Menu.Label>
        {choices.map(({ value, icon: Icon }) => (
          <Menu.Item
            key={value}
            leftSection={<Icon size={16} />}
            rightSection={colorScheme === value ? <IconCheck size={14} /> : undefined}
            onClick={() => setColorScheme(value)}
          >
            {t(`settings.schemes.${value}`)}
          </Menu.Item>
        ))}
      </Menu.Dropdown>
    </Menu>
  );
}
