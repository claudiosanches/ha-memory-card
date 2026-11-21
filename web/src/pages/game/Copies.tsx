import { ActionIcon, Badge, Group, Table, Text } from '@mantine/core';
import { IconCheck, IconPencil, IconTrash } from '@tabler/icons-react';
import { useMediaQuery } from '@mantine/hooks';
import { useTranslation } from 'react-i18next';
import type { CopyDetail } from '../../api/types';
import classes from './Copies.module.css';

type Props = {
  copies: CopyDetail[];
  devices: Map<number, string>;
  stores: Map<number, string>;
  /** Without handlers, the copies are read only. */
  onEdit?: (copy: CopyDetail) => void;
  onDelete?: (copy: CopyDetail) => void;
  /** Moves a wanted copy into the collection. */
  onGotIt?: (copy: CopyDetail) => void;
};

export function Copies({ copies, devices, stores, onEdit, onDelete, onGotIt }: Props) {
  const { t } = useTranslation();
  const phone = useMediaQuery('(max-width: 48em)');
  const device = (copy: CopyDetail) => (copy.device_id ? devices.get(copy.device_id) : null);
  const store = (copy: CopyDetail) => (copy.store_id ? stores.get(copy.store_id) : null);
  const condition = (copy: CopyDetail) => (copy.condition ? t(`condition.${copy.condition}`) : null);
  const editable = onEdit && onDelete;
  const anyWanted = copies.some((copy) => copy.status === 'wanted');
  const wanted = (copy: CopyDetail) =>
    copy.status === 'wanted' && (
      <Badge size="xs" variant="light" color="pink" ml={6}>
        {t('copyStatus.wanted')}
      </Badge>
    );
  const actions = (copy: CopyDetail) => editable && (
    <Group gap={4} wrap="nowrap" className={classes.actions}>
      {onGotIt && copy.status === 'wanted' && (
        <ActionIcon variant="subtle" color="green" aria-label={t('game.gotIt')} onClick={() => onGotIt(copy)}>
          <IconCheck size={16} />
        </ActionIcon>
      )}
      <ActionIcon variant="subtle" aria-label={t('game.editCopy')} onClick={() => onEdit(copy)}>
        <IconPencil size={16} />
      </ActionIcon>
      <ActionIcon variant="subtle" color="red" aria-label={t('game.deleteCopy')} onClick={() => onDelete(copy)}>
        <IconTrash size={16} />
      </ActionIcon>
    </Group>
  );

  const line = (parts: (string | null | undefined)[]) => parts.filter(Boolean).join(t('common.separator'));
  if (phone) {
    return (
    <ul className={classes.list}>
      {copies.map((copy) => {
        const where = line([device(copy), store(copy)]);
        return (
          <li key={copy.id} className={classes.copy} data-testid={`copy-${copy.id}`}>
              <div className={classes.details}>
                <Text size="sm" fw={500}>
                  {line([t(`format.${copy.format}`), condition(copy)])}
                  {wanted(copy)}
                </Text>
                {where && (
                  <Text size="xs" c="dimmed">
                    {where}
                  </Text>
                )}
                {copy.notes && (
                  <Text size="xs" c="dimmed" lineClamp={2}>
                    {copy.notes}
                  </Text>
                )}
              </div>
              {actions(copy)}
            </li>
        );
      })}
    </ul>
    );
  }

  return (
    <Table className={classes.table}>
      <colgroup>
        <col className={classes.formatColumn} />
        <col className={classes.deviceColumn} />
        <col className={classes.storeColumn} />
        <col className={classes.conditionColumn} />
        <col />
        {editable && <col className={anyWanted ? classes.actionsColumnWide : classes.actionsColumn} />}
      </colgroup>
      <Table.Thead>
        <Table.Tr>
          <Table.Th>{t('game.format')}</Table.Th>
          <Table.Th>{t('game.device')}</Table.Th>
          <Table.Th>{t('game.store')}</Table.Th>
          <Table.Th>{t('game.condition')}</Table.Th>
          <Table.Th>{t('common.notes')}</Table.Th>
          {editable && <Table.Th aria-label={t('common.actions')} />}
        </Table.Tr>
      </Table.Thead>
      <Table.Tbody>
        {copies.map((copy) => (
          <Table.Tr key={copy.id} data-testid={`copy-${copy.id}`}>
            <Table.Td>
              {t(`format.${copy.format}`)}
              {wanted(copy)}
            </Table.Td>
            <Table.Td>{device(copy)}</Table.Td>
            <Table.Td>{store(copy)}</Table.Td>
            <Table.Td>{condition(copy)}</Table.Td>
            <Table.Td><Text size="sm" className={classes.notes}>{copy.notes}</Text></Table.Td>
            {editable && <Table.Td>{actions(copy)}</Table.Td>}
          </Table.Tr>
        ))}
      </Table.Tbody>
    </Table>
  );
}
