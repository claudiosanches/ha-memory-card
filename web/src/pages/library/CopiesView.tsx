import { Table, Text } from '@mantine/core';
import { useMediaQuery } from '@mantine/hooks';
import { Link } from '@tanstack/react-router';
import { useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useDevices, useStores } from '../../api/queries';
import type { CopyRow, Platform } from '../../api/types';
import { Cover } from '../../components/Cover';
import classes from './CopiesView.module.css';

type Props = { copies: CopyRow[]; platformsById: Map<number, Platform> };

/** One row per copy, with its platform, region and where it lives. */
export function CopiesView({ copies, platformsById }: Props) {
  const { t } = useTranslation();
  const phone = useMediaQuery('(max-width: 48em)');
  const devices = useDevices();
  const stores = useStores();
  const deviceNames = useMemo(() => new Map((devices.data ?? []).map((d) => [d.id, d.name])), [devices.data]);
  const storeNames = useMemo(() => new Map((stores.data ?? []).map((s) => [s.id, s.name])), [stores.data]);

  const describe = (copy: CopyRow) => ({
    platform: platformsById.get(copy.platform_id)?.name ?? '',
    region: t(`region.${copy.region}`),
    format: t(`format.${copy.format}`),
    condition: copy.condition ? t(`condition.${copy.condition}`) : null,
    device: copy.device_id ? (deviceNames.get(copy.device_id) ?? null) : null,
    store: copy.store_id ? (storeNames.get(copy.store_id) ?? null) : null,
  });
  const line = (parts: (string | null)[]) => parts.filter(Boolean).join(t('common.separator'));

  const title = (copy: CopyRow) => (
    <>
      <Link to="/games/$gameId" params={{ gameId: String(copy.game_id) }} className={classes.title}>
        {copy.title}
      </Link>
      {copy.release_title && copy.release_title !== copy.title && (
        <Text size="xs" c="dimmed" lineClamp={1}>
          {copy.release_title}
        </Text>
      )}
    </>
  );

  if (phone) {
    return (
      <ul className={classes.list}>
        {copies.map((copy) => {
          const d = describe(copy);
          const where = line([d.device, d.store]);
          return (
            <li key={copy.copy_id} className={classes.item} data-testid={`copy-row-${copy.copy_id}`}>
              <div className={classes.cover}>
                <Cover path={copy.cover_path} title={copy.title} />
              </div>
              <div className={classes.details}>
                {title(copy)}
                <Text size="sm">{line([d.platform, d.region])}</Text>
                <Text size="xs" c="dimmed">
                  {line([d.format, d.condition])}
                </Text>
                {where && (
                  <Text size="xs" c="dimmed">
                    {where}
                  </Text>
                )}
              </div>
            </li>
          );
        })}
      </ul>
    );
  }

  return (
    <div className={classes.panel}>
      <Table className={classes.table} highlightOnHover>
        <colgroup>
          <col className={classes.coverColumn} />
          <col />
          <col className={classes.platformColumn} />
          <col className={classes.shortColumn} />
          <col className={classes.shortColumn} />
          <col className={classes.shortColumn} />
          <col className={classes.whereColumn} />
          <col className={classes.whereColumn} />
        </colgroup>
        <Table.Thead>
          <Table.Tr>
            <Table.Th aria-label={t('cover.label')} />
            <Table.Th>{t('game.title')}</Table.Th>
            <Table.Th>{t('game.platform')}</Table.Th>
            <Table.Th>{t('game.region')}</Table.Th>
            <Table.Th>{t('game.format')}</Table.Th>
            <Table.Th>{t('game.condition')}</Table.Th>
            <Table.Th>{t('game.device')}</Table.Th>
            <Table.Th>{t('game.store')}</Table.Th>
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {copies.map((copy) => {
            const d = describe(copy);
            return (
              <Table.Tr key={copy.copy_id} data-testid={`copy-row-${copy.copy_id}`}>
                <Table.Td>
                  <Cover path={copy.cover_path} title={copy.title} width={36} />
                </Table.Td>
                <Table.Td>{title(copy)}</Table.Td>
                <Table.Td>{d.platform}</Table.Td>
                <Table.Td>{d.region}</Table.Td>
                <Table.Td>{d.format}</Table.Td>
                <Table.Td>{d.condition}</Table.Td>
                <Table.Td>{d.device}</Table.Td>
                <Table.Td>{d.store}</Table.Td>
              </Table.Tr>
            );
          })}
        </Table.Tbody>
      </Table>
    </div>
  );
}
