import { ActionIcon, Group, Table, Text } from '@mantine/core';
import { IconPencil, IconTrash } from '@tabler/icons-react';
import { useTranslation } from 'react-i18next';
import type { CopyDetail } from '../../api/types';
import classes from './Copies.module.css';

type Props = {
  copies: CopyDetail[];
  devices: Map<number, string>;
  stores: Map<number, string>;
  /** A stacked list for screens where the table would scroll sideways. */
  compact: boolean;
  onEdit: (copy: CopyDetail) => void;
  onDelete: (copy: CopyDetail) => void;
};

export function Copies({ copies, devices, stores, compact, onEdit, onDelete }: Props) {
  const { t } = useTranslation();
  const device = (copy: CopyDetail) => (copy.device_id ? devices.get(copy.device_id) : null);
  const store = (copy: CopyDetail) => (copy.store_id ? stores.get(copy.store_id) : null);
  const condition = (copy: CopyDetail) => (copy.condition ? t(`condition.${copy.condition}`) : null);
  const actions = (copy: CopyDetail) => (
    <Group gap={4} wrap="nowrap" className={classes.actions}>
      <ActionIcon variant="subtle" aria-label={t('game.editCopy')} onClick={() => onEdit(copy)}>
        <IconPencil size={16} />
      </ActionIcon>
      <ActionIcon variant="subtle" color="red" aria-label={t('game.deleteCopy')} onClick={() => onDelete(copy)}>
        <IconTrash size={16} />
      </ActionIcon>
    </Group>
  );

  if (compact) {
    const line = (parts: (string | null | undefined)[]) => parts.filter(Boolean).join(t('common.separator'));
    return (
      <ul className={classes.list}>
        {copies.map((copy) => {
          const where = line([device(copy), store(copy)]);
          return (
            <li key={copy.id} className={classes.copy} data-testid={`copy-${copy.id}`}>
              <div className={classes.details}>
                <Text size="sm" fw={500}>
                  {line([t(`format.${copy.format}`), condition(copy)])}
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
    <Table.ScrollContainer minWidth={560}>
      <Table>
        <Table.Thead>
          <Table.Tr>
            <Table.Th>{t('game.format')}</Table.Th>
            <Table.Th>{t('game.device')}</Table.Th>
            <Table.Th>{t('game.store')}</Table.Th>
            <Table.Th>{t('game.condition')}</Table.Th>
            <Table.Th>{t('common.notes')}</Table.Th>
            <Table.Th w={80} />
          </Table.Tr>
        </Table.Thead>
        <Table.Tbody>
          {copies.map((copy) => (
            <Table.Tr key={copy.id} data-testid={`copy-${copy.id}`}>
              <Table.Td>{t(`format.${copy.format}`)}</Table.Td>
              <Table.Td>{device(copy)}</Table.Td>
              <Table.Td>{store(copy)}</Table.Td>
              <Table.Td>{condition(copy)}</Table.Td>
              <Table.Td>
                <Text size="sm" lineClamp={2}>
                  {copy.notes}
                </Text>
              </Table.Td>
              <Table.Td>{actions(copy)}</Table.Td>
            </Table.Tr>
          ))}
        </Table.Tbody>
      </Table>
    </Table.ScrollContainer>
  );
}
