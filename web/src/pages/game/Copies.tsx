import { ActionIcon, Badge, Group, Text } from '@mantine/core';
import { IconCheck, IconPencil, IconTrash } from '@tabler/icons-react';
import { Fragment, type ReactNode } from 'react';
import { useTranslation } from 'react-i18next';
import type { CopyDetail } from '../../api/types';
import { history } from '../../components/history';
import { formatMoney } from '../../components/money';
import { formatDate } from '../../components/status';
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

/** Parts in their own elements, joined by the separator. */
function Parts({ parts }: { parts: ReactNode[] }) {
  const { t } = useTranslation();
  const shown = parts.filter(Boolean);
  return shown.map((part, i) => (
    <Fragment key={i}>
      {i > 0 && t('common.separator')}
      <span>{part}</span>
    </Fragment>
  ));
}

export function Copies({ copies, devices, stores, onEdit, onDelete, onGotIt }: Props) {
  const { t, i18n } = useTranslation();
  const editable = onEdit && onDelete;

  return (
    <ul className={classes.list}>
      {copies.map((copy) => {
        const wanted = copy.status === 'wanted';
        const store = copy.store_id ? stores.get(copy.store_id) : null;
        const where = [
          copy.device_id ? devices.get(copy.device_id) : null,
          store && (wanted ? t('wishlist.store', { store }) : store),
          copy.purchased_on && t('game.boughtOn', { date: formatDate(copy.purchased_on, i18n.language) }),
          copy.price && copy.currency && formatMoney(copy.price, copy.currency, i18n.language),
        ];
        return (
          <li key={copy.id} className={classes.copy} data-testid={`copy-${copy.id}`}>
            <div className={classes.details}>
              <Text size="sm" fw={500}>
                <Parts
                  parts={[t(`format.${copy.format}`), copy.edition, copy.condition && t(`condition.${copy.condition}`)]}
                />
                {wanted && (
                  <Badge size="xs" variant="light" color="pink" ml={6} style={{ overflow: 'visible' }}>
                    {t('copyStatus.badge')}
                  </Badge>
                )}
              </Text>
              {where.some(Boolean) && (
                <Text size="xs" c="dimmed">
                  <Parts parts={where} />
                </Text>
              )}
              {copy.notes && (
                <Text size="xs" c="dimmed" className={classes.notes}>
                  {copy.notes}
                </Text>
              )}
              <Text size="xs" c="dimmed" className={classes.history}>
                {history(copy, t, i18n.language)}
              </Text>
            </div>
            {editable && (
              <Group gap={4} wrap="nowrap" className={classes.actions}>
                {onGotIt && wanted && (
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
            )}
          </li>
        );
      })}
    </ul>
  );
}
