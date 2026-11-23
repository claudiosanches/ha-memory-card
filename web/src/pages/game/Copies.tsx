import { ActionIcon, Anchor, Badge, Group, Menu, Text } from '@mantine/core';
import { IconCamera, IconCheck, IconDots, IconNfc, IconNfcOff, IconPackages, IconPencil, IconTrash } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { Fragment, type ReactNode, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import type { CopyDetail, GameDetail } from '../../api/types';
import { history } from '../../components/history';
import { formatMoney } from '../../components/money';
import { notifyError } from '../../components/notify';
import { useFormatDate } from '../../hooks/useFormatDate';
import classes from './Copies.module.css';
import { useGameSaved } from './forms';
import { PhotoViewer } from './PhotoViewer';

type Props = {
  copies: CopyDetail[];
  devices: Map<number, string>;
  stores: Map<number, string>;
  /** Without handlers, the copies are read only. */
  onEdit?: (copy: CopyDetail) => void;
  onDelete?: (copy: CopyDetail) => void;
  /** Moves a wanted copy into the collection. */
  onGotIt?: (copy: CopyDetail) => void;
  /** Adds games that came inside this copy. */
  onAddContents?: (copy: CopyDetail) => void;
  /** Waits for a Home Assistant tag scan to link; only while Memory Card hears scans. */
  onLinkTag?: (copy: CopyDetail) => void;
};

const PHOTO_TYPES = 'image/jpeg,image/png,image/webp';

function GameLink({ id, title }: { id: number; title: string }) {
  return (
    <Anchor inherit renderRoot={(props) => <Link to="/games/$gameId" params={{ gameId: String(id) }} {...props} />}>
      {title}
    </Anchor>
  );
}

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

export function Copies(props: Props) {
  return (
    <ul className={classes.list}>
      {props.copies.map((copy) => (
        <CopyItem key={copy.id} {...props} copy={copy} />
      ))}
    </ul>
  );
}

function CopyItem({ copy, devices, stores, onEdit, onDelete, onGotIt, onAddContents, onLinkTag }: Props & { copy: CopyDetail }) {
  const { t, i18n } = useTranslation();
  const formatDate = useFormatDate();
  const saved = useGameSaved();
  const input = useRef<HTMLInputElement>(null);
  const [viewing, setViewing] = useState<number | null>(null);
  const editable = onEdit && onDelete;

  // The server reads the image type from the bytes, so the body goes as is.
  const upload = useMutation({
    mutationFn: async (files: File[]) => {
      let latest: GameDetail | null = null;
      for (const file of files) {
        latest = await call(
          api.POST('/api/copies/{id}/photos', {
            params: { path: { id: copy.id } },
            body: file as unknown as string,
            bodySerializer: (body: unknown) => body as BodyInit,
            headers: { 'Content-Type': 'application/octet-stream' },
          }),
        );
      }
      return latest;
    },
    onSuccess: (game) => game && saved(game, t('photos.added')),
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const unlinkTags = useMutation({
    mutationFn: async () => {
      let latest: GameDetail | null = null;
      for (const tag of copy.tags) {
        latest = await call(api.DELETE('/api/tags/{id}', { params: { path: { id: tag.tag_id } } }));
      }
      return latest;
    },
    onSuccess: (game) => game && saved(game, t('tags.unlinked')),
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const wanted = copy.status === 'wanted';
  const store = copy.store_id ? stores.get(copy.store_id) : null;
  const where = [
    copy.device_id ? devices.get(copy.device_id) : null,
    store && (wanted ? t('wishlist.store', { store }) : store),
    copy.purchased_on && t('game.boughtOn', { date: formatDate(copy.purchased_on) }),
    copy.price && copy.currency && formatMoney(copy.price, copy.currency, i18n.language),
    copy.estimated_value &&
      copy.estimated_currency &&
      t('game.worth', { amount: formatMoney(copy.estimated_value, copy.estimated_currency, i18n.language) }),
  ];

  return (
    <li className={classes.copy} data-testid={`copy-${copy.id}`}>
      <div className={classes.details}>
        <Text size="sm" fw={500}>
          <Parts parts={[t(`format.${copy.format}`), copy.edition, copy.condition && t(`condition.${copy.condition}`)]} />
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
        {copy.bundle && (
          <Text size="xs" data-testid={`bundle-${copy.id}`}>
            {t('bundle.partOf')} <GameLink id={copy.bundle.game_id} title={copy.bundle.title} />
          </Text>
        )}
        {copy.contents.length > 0 && (
          <Text size="xs" data-testid={`contents-${copy.id}`}>
            {t('bundle.includes')}{' '}
            {copy.contents.map((item, i) => (
              <Fragment key={item.copy_id}>
                {i > 0 && ', '}
                <GameLink id={item.game_id} title={item.title} />
              </Fragment>
            ))}
          </Text>
        )}
        {copy.notes && (
          <Text size="xs" c="dimmed" className={classes.notes}>
            {copy.notes}
          </Text>
        )}
        {copy.tags.length > 0 && (
          <Text size="xs" c="dimmed" data-testid={`tags-${copy.id}`}>
            <IconNfc size={12} style={{ verticalAlign: 'middle', marginRight: 4 }} aria-hidden />
            {t('tags.count', { count: copy.tags.length })}
          </Text>
        )}
        {copy.photos.length > 0 && (
          <div className={classes.photos} data-testid={`photos-${copy.id}`}>
            {copy.photos.map((photo, i) => (
              <button
                key={photo.id}
                type="button"
                className={classes.thumb}
                aria-label={t('photos.open', { number: i + 1 })}
                onClick={() => setViewing(i)}
              >
                <img src={`covers/${photo.path}`} alt="" loading="lazy" decoding="async" />
              </button>
            ))}
          </div>
        )}
        <Text size="xs" c="dimmed" className={classes.history}>
          {history(copy, t, formatDate)}
          {copy.source === 'csv' && ` ${t('history.fromCsv')}`}
        </Text>
      </div>

      {editable && copy.bundle && (
        <Group gap={4} wrap="nowrap" className={classes.actions}>
          <ActionIcon variant="subtle" color="red" aria-label={t('bundle.remove')} onClick={() => onDelete(copy)}>
            <IconTrash size={16} />
          </ActionIcon>
        </Group>
      )}
      {editable && !copy.bundle && (
        <Group gap={4} wrap="nowrap" className={classes.actions}>
          {onGotIt && wanted && (
            <ActionIcon variant="subtle" color="green" aria-label={t('game.gotIt')} onClick={() => onGotIt(copy)}>
              <IconCheck size={16} />
            </ActionIcon>
          )}
          <ActionIcon variant="subtle" aria-label={t('game.editCopy')} onClick={() => onEdit(copy)}>
            <IconPencil size={16} />
          </ActionIcon>
          <Menu position="bottom-end" withinPortal>
            <Menu.Target>
              <ActionIcon variant="subtle" aria-label={t('game.moreCopyActions')} loading={upload.isPending}>
                <IconDots size={16} />
              </ActionIcon>
            </Menu.Target>
            <Menu.Dropdown>
              <Menu.Item leftSection={<IconCamera size={16} />} onClick={() => input.current?.click()}>
                {t('photos.add')}
              </Menu.Item>
              {onAddContents && (
                <Menu.Item leftSection={<IconPackages size={16} />} onClick={() => onAddContents(copy)}>
                  {t('bundle.add')}
                </Menu.Item>
              )}
              {onLinkTag && !wanted && (
                <Menu.Item leftSection={<IconNfc size={16} />} onClick={() => onLinkTag(copy)}>
                  {t('tags.link')}
                </Menu.Item>
              )}
              {copy.tags.length > 0 && (
                <Menu.Item leftSection={<IconNfcOff size={16} />} onClick={() => unlinkTags.mutate()}>
                  {t('tags.unlink')}
                </Menu.Item>
              )}
              <Menu.Item color="red" leftSection={<IconTrash size={16} />} onClick={() => onDelete(copy)}>
                {t('game.deleteCopyAction')}
              </Menu.Item>
            </Menu.Dropdown>
          </Menu>
          <input
            ref={input}
            type="file"
            accept={PHOTO_TYPES}
            multiple
            hidden
            data-testid={`photo-input-${copy.id}`}
            onChange={(event) => {
              const files = [...(event.currentTarget.files ?? [])];
              event.currentTarget.value = '';
              if (files.length > 0) upload.mutate(files);
            }}
          />
        </Group>
      )}

      {viewing != null && (
        <PhotoViewer photos={copy.photos} start={viewing} canDelete={!!editable} onClose={() => setViewing(null)} />
      )}
    </li>
  );
}
