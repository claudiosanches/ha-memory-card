import { Center, Image } from '@mantine/core';
import { IconDeviceGamepad2 } from '@tabler/icons-react';
import { useTranslation } from 'react-i18next';

/** Game cover at a 3:4 ratio, or a placeholder when there is none. */
export function Cover({ path, title, width }: { path?: string | null; title: string; width: number }) {
  const { t } = useTranslation();
  const height = Math.round((width * 4) / 3);
  if (!path) {
    return (
      <Center w={width} h={height} bg="var(--mantine-color-default-hover)" style={{ borderRadius: 4, flexShrink: 0 }}>
        <IconDeviceGamepad2 size={width / 3} color="var(--mantine-color-dimmed)" />
      </Center>
    );
  }
  // Relative, so it resolves under the ingress path.
  return (
    <Image
      src={`covers/${path}`}
      alt={t('game.cover', { title })}
      w={width}
      h={height}
      radius={4}
      fit="cover"
      style={{ flexShrink: 0 }}
    />
  );
}
