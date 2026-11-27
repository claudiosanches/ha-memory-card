import {
  ActionIcon,
  Badge,
  Button,
  Card,
  Group,
  Loader,
  Select,
  Stack,
  Text,
  TextInput,
  Tooltip,
} from '@mantine/core';
import { IconCamera, IconPencil, IconPlus, IconSearch, IconTrash, IconX } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { Link } from '@tanstack/react-router';
import { useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useCanManage, useDevices, usePlatforms, useStores } from '../../api/queries';
import { type CopyStatus, type Device, type DeviceKind, DEVICE_KINDS, holdsGames } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { EmptyState } from '../../components/EmptyState';
import { FilterBar, FilterButton } from '../../components/FilterBar';
import { formatMoney } from '../../components/money';
import { notifyError, notifySuccess } from '../../components/notify';
import { PageHeader } from '../../components/PageHeader';
import { useFormatDate } from '../../hooks/useFormatDate';
import { PhotoViewer } from '../game/PhotoViewer';
import { DEVICE_KIND_ICON, HARDWARE_NAV_ICON } from './deviceIcons';
import { HardwareForm } from './HardwareForm';
import classes from './HardwarePage.module.css';

const PHOTO_TYPES = 'image/jpeg,image/png,image/webp';

type Show = CopyStatus | 'all';

/** The household's hardware: consoles, handhelds and PCs copies live on, and controllers, accessories and toys. */
export function HardwarePage() {
  const { t } = useTranslation();
  const devices = useDevices();
  const platforms = usePlatforms();
  const canManage = useCanManage();
  const [show, setShow] = useState<Show>('owned');
  const [kind, setKind] = useState<DeviceKind | null>(null);
  const [query, setQuery] = useState('');
  const [editing, setEditing] = useState<{ device?: Device } | null>(null);
  const [deleting, setDeleting] = useState<Device | null>(null);
  const queryClient = useQueryClient();

  const remove = useMutation({
    mutationFn: (id: number) => call(api.DELETE('/api/devices/{id}', { params: { path: { id } } })),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.devices });
      void queryClient.invalidateQueries({ queryKey: queryKeys.platforms });
      void queryClient.invalidateQueries({ queryKey: queryKeys.stats });
      setDeleting(null);
      notifySuccess(t('common.deleted'));
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });

  const platformSearchText = useMemo(() => {
    const map = new Map<number, string>();
    for (const p of platforms.data ?? []) {
      map.set(p.id, [p.name, p.abbreviation].filter(Boolean).join(' ').toLowerCase());
    }
    return map;
  }, [platforms.data]);

  const needle = query.trim().toLowerCase();

  // Filtered by ownership and search, before the kind tab filter, so kind counts stay accurate.
  const matchingStatusAndQuery = useMemo(() => {
    return (devices.data ?? []).filter((d) => {
      if (show !== 'all' && d.status !== show) return false;
      if (!needle) return true;
      const pids = holdsGames(d.kind) ? d.platform_ids : d.platform_id ? [d.platform_id] : [];
      const haystack = [
        d.name,
        d.model,
        d.notes,
        d.serial_number,
        d.barcode,
        t(`deviceKind.${d.kind}`),
        ...pids.map((id) => platformSearchText.get(id) ?? ''),
      ]
        .filter(Boolean)
        .join(' ')
        .toLowerCase();
      return haystack.includes(needle);
    });
  }, [devices.data, show, needle, platformSearchText, t]);

  const kindCounts = useMemo(() => {
    const counts = new Map<DeviceKind, number>();
    for (const d of matchingStatusAndQuery) {
      counts.set(d.kind, (counts.get(d.kind) ?? 0) + 1);
    }
    return counts;
  }, [matchingStatusAndQuery]);

  const visibleKinds = DEVICE_KINDS.filter((k) => (kindCounts.get(k) ?? 0) > 0 || kind === k);

  const shown = useMemo(
    () => (kind ? matchingStatusAndQuery.filter((d) => d.kind === kind) : matchingStatusAndQuery),
    [matchingStatusAndQuery, kind],
  );

  const groups = useMemo(() => {
    const byKind = new Map<DeviceKind, Device[]>();
    for (const device of shown) {
      const list = byKind.get(device.kind);
      if (list) list.push(device);
      else byKind.set(device.kind, [device]);
    }
    return DEVICE_KINDS.filter((k) => byKind.has(k)).map((k) => ({ kind: k, items: byKind.get(k)! }));
  }, [shown]);

  const showGroups = !kind && groups.length > 1;
  const hasFilter = Boolean(needle || kind);

  const renderCard = (device: Device) => (
    <HardwareCard
      key={device.id}
      device={device}
      canManage={canManage}
      onEdit={(status) => setEditing({ device: status ? { ...device, status } : device })}
      onDelete={() => setDeleting(device)}
    />
  );

  return (
    <Stack gap="section">
      <PageHeader
        title={t('nav.hardware')}
        description={t('hardware.intro')}
        count={devices.data && t('hardware.count', { count: shown.length })}
        countTestId="hardware-count"
      >
        {canManage && (
          <Button leftSection={<IconPlus size={18} />} onClick={() => setEditing({})}>
            {t('hardware.add')}
          </Button>
        )}
      </PageHeader>

      <div className={classes.controls}>
        <div className={classes.toolbar}>
          <TextInput
            className={classes.search}
            size="md"
            leftSection={<IconSearch size={18} />}
            placeholder={t('hardware.searchPlaceholder')}
            aria-label={t('hardware.searchPlaceholder')}
            value={query}
            rightSection={
              query ? (
                <ActionIcon
                  variant="subtle"
                  color="gray"
                  size="sm"
                  aria-label={t('library.clearSearch')}
                  onClick={() => setQuery('')}
                >
                  <IconX size={16} />
                </ActionIcon>
              ) : null
            }
            onChange={(event) => setQuery(event.currentTarget.value)}
          />
          <Select
            size="md"
            className={classes.ownership}
            aria-label={t('hardware.show')}
            value={show}
            onChange={(value) => value && setShow(value as Show)}
            allowDeselect={false}
            data={[
              { value: 'owned', label: t('hardware.showOwned') },
              { value: 'wanted', label: t('nav.wishlist') },
              { value: 'all', label: t('hardware.showAll') },
            ]}
          />
        </div>

        {visibleKinds.length > 0 && (
          <FilterBar label={t('devices.kind')}>
            <FilterButton selected={kind === null} onClick={() => setKind(null)}>
              {t('hardware.allKinds')}
            </FilterButton>
            {visibleKinds.map((k) => (
              <FilterButton key={k} selected={kind === k} onClick={() => setKind(k)}>
                {t(`deviceKind.${k}`)}
              </FilterButton>
            ))}
          </FilterBar>
        )}
      </div>

      {devices.isPending || platforms.isPending ? (
        <Loader size="sm" />
      ) : devices.isError ? (
        <Text c="red">{errorMessage(t, devices.error)}</Text>
      ) : shown.length === 0 ? (
        <EmptyState
          icon={HARDWARE_NAV_ICON}
          title={
            hasFilter
              ? t('hardware.noMatchesTitle')
              : show === 'wanted'
                ? t('hardware.emptyWantedTitle')
                : t('hardware.emptyTitle')
          }
          description={
            hasFilter
              ? t('hardware.noMatches')
              : show === 'wanted'
                ? t(canManage ? 'hardware.emptyWanted' : 'hardware.emptyWantedViewer')
                : t(canManage ? 'hardware.empty' : 'hardware.emptyViewer')
          }
          action={
            hasFilter ? (
              <Button
                variant="default"
                onClick={() => {
                  setQuery('');
                  setKind(null);
                }}
              >
                {t('library.clearFilters')}
              </Button>
            ) : undefined
          }
        />
      ) : showGroups ? (
        <div className={classes.groups}>
          {groups.map(({ kind: groupKind, items }) => {
            const GroupIcon = DEVICE_KIND_ICON[groupKind];
            return (
              <section key={groupKind} className={classes.group}>
                <div className={classes.groupHeader}>
                  <span className={classes.groupIcon} aria-hidden>
                    <GroupIcon size={17} stroke={1.7} />
                  </span>
                  <Text className={classes.groupTitle}>{t(`deviceKind.${groupKind}`)}</Text>
                  <span className={classes.groupCount}>{items.length}</span>
                </div>
                <ul className={classes.grid}>{items.map(renderCard)}</ul>
              </section>
            );
          })}
        </div>
      ) : (
        <ul className={classes.grid}>{shown.map(renderCard)}</ul>
      )}

      {editing && <HardwareForm device={editing.device} onClose={() => setEditing(null)} />}

      <ConfirmModal
        opened={deleting !== null}
        title={t('devices.deleteTitle', { name: deleting?.name ?? '' })}
        message={deleting?.copy_count ? t('devices.deleteCopies', { count: deleting.copy_count }) : undefined}
        confirmLabel={t('common.delete')}
        busy={remove.isPending}
        onConfirm={() => deleting && remove.mutate(deleting.id)}
        onClose={() => setDeleting(null)}
      />
    </Stack>
  );
}

function HardwareCard({
  device,
  canManage,
  onEdit,
  onDelete,
}: {
  device: Device;
  canManage: boolean;
  onEdit: (status?: CopyStatus) => void;
  onDelete: () => void;
}) {
  const { t, i18n } = useTranslation();
  const formatDate = useFormatDate();
  const platforms = usePlatforms();
  const stores = useStores();
  const queryClient = useQueryClient();
  const input = useRef<HTMLInputElement>(null);
  const [viewing, setViewing] = useState<number | null>(null);
  const platformName = useMemo(
    () => new Map((platforms.data ?? []).map((p) => [p.id, p.abbreviation ?? p.name])),
    [platforms.data],
  );
  const wanted = device.status === 'wanted';
  const refresh = () => void queryClient.invalidateQueries({ queryKey: queryKeys.devices });

  // The server reads the image type from the bytes, so the body goes as is.
  const upload = useMutation({
    mutationFn: async (files: File[]) => {
      for (const file of files) {
        await call(
          api.POST('/api/devices/{id}/photos', {
            params: { path: { id: device.id } },
            body: file as unknown as string,
            bodySerializer: (body: unknown) => body as BodyInit,
            headers: { 'Content-Type': 'application/octet-stream' },
          }),
        );
      }
    },
    onSuccess: () => {
      refresh();
      notifySuccess(t('photos.added'));
    },
    onError: (error) => {
      refresh();
      notifyError(errorMessage(t, error));
    },
  });

  const store = device.store_id ? stores.data?.find((s) => s.id === device.store_id)?.name : null;
  const money = (amount?: string | null, currency?: string | null) =>
    amount && currency ? formatMoney(amount, currency, i18n.language) : null;
  const price = money(device.price, device.currency);
  const value = money(device.estimated_value, device.estimated_currency);
  const details = [
    device.condition && t(`condition.${device.condition}`),
    device.region && t(`region.${device.region}`),
    store && (wanted ? t('wishlist.store', { store }) : store),
    device.purchased_on && t('game.boughtOn', { date: formatDate(device.purchased_on) }),
  ].filter(Boolean);
  const prices = [
    price && (wanted ? t('hardware.expected', { amount: price }) : t('hardware.paid', { amount: price })),
    value && t('hardware.worth', { amount: value }),
  ].filter(Boolean);
  const Icon = DEVICE_KIND_ICON[device.kind];
  const platformIds = holdsGames(device.kind) ? device.platform_ids : device.platform_id ? [device.platform_id] : [];
  const hasBody = details.length > 0 || prices.length > 0 || Boolean(device.notes) || device.photos.length > 0;
  const hasFooter = platformIds.length > 0 || holdsGames(device.kind);

  return (
    <Card component="li" withBorder padding="md" className={classes.card} data-testid={`device-${device.name}`}>
      <div className={classes.header}>
        <div className={classes.emblem} aria-hidden>
          <Icon size={20} stroke={1.6} />
        </div>
        <div className={classes.heading}>
          <div className={classes.titleRow}>
            <Text className={classes.name}>{device.name}</Text>
            {wanted && (
              <Badge size="xs" variant="light">
                {t('copyStatus.badge')}
              </Badge>
            )}
            {!device.active && (
              <Badge size="xs" color="gray" variant="light">
                {t('common.inactive')}
              </Badge>
            )}
          </div>
          <Text size="xs" c="dimmed">
            {[t(`deviceKind.${device.kind}`), device.model].filter(Boolean).join(t('common.separator'))}
          </Text>
        </div>
        {canManage && (
          <Group gap={2} wrap="nowrap" className={classes.actions}>
            {wanted && (
              <Button size="compact-xs" variant="light" color="green" onClick={() => onEdit('owned')}>
                {t('game.gotIt')}
              </Button>
            )}
            <Tooltip label={t('photos.add')}>
              <ActionIcon
                variant="subtle"
                color="gray"
                className={classes.actionBtn}
                aria-label={t('photos.add')}
                loading={upload.isPending}
                onClick={() => input.current?.click()}
              >
                <IconCamera size={16} />
              </ActionIcon>
            </Tooltip>
            <Tooltip label={t('common.edit')}>
              <ActionIcon
                variant="subtle"
                color="gray"
                className={classes.actionBtn}
                aria-label={t('common.edit')}
                onClick={() => onEdit()}
              >
                <IconPencil size={16} />
              </ActionIcon>
            </Tooltip>
            <Tooltip label={t('common.delete')}>
              <ActionIcon
                variant="subtle"
                color="gray"
                className={classes.deleteBtn}
                aria-label={t('common.delete')}
                onClick={onDelete}
              >
                <IconTrash size={16} />
              </ActionIcon>
            </Tooltip>
            <input
              ref={input}
              type="file"
              accept={PHOTO_TYPES}
              multiple
              hidden
              data-testid={`device-photo-input-${device.id}`}
              onChange={(event) => {
                const files = [...(event.currentTarget.files ?? [])];
                event.currentTarget.value = '';
                if (files.length > 0) upload.mutate(files);
              }}
            />
          </Group>
        )}
      </div>

      {hasBody && (
        <div className={classes.body}>
          {details.length > 0 && (
            <Text size="xs" c="dimmed">
              {details.join(t('common.separator'))}
            </Text>
          )}
          {prices.length > 0 && (
            <Text size="xs" className={classes.prices}>
              {prices.join(t('common.separator'))}
            </Text>
          )}
          {device.notes && (
            <Text size="xs" c="dimmed" className={classes.notes}>
              {device.notes}
            </Text>
          )}
          {device.photos.length > 0 && (
            <div className={classes.photos}>
              {device.photos.map((photo, i) => (
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
        </div>
      )}

      {hasFooter && (
        <div className={classes.footer}>
          <div className={classes.platforms}>
            {platformIds.map((id) => (
              <span key={id} className={classes.platformLabel}>
                {platformName.get(id)}
              </span>
            ))}
          </div>
          {holdsGames(device.kind) &&
            (device.copy_count > 0 ? (
              <Link to="/" search={{ device: [device.slug] }} className={classes.copyLink}>
                {t('common.copyCount', { count: device.copy_count })}
              </Link>
            ) : (
              <Text size="xs" c="dimmed" className={classes.count}>
                {t('common.copyCount', { count: device.copy_count })}
              </Text>
            ))}
        </div>
      )}

      {viewing != null && (
        <PhotoViewer
          photos={device.photos}
          start={viewing}
          canDelete={canManage}
          onClose={() => setViewing(null)}
          onDelete={async (id) => {
            await call(api.DELETE('/api/device-photos/{id}', { params: { path: { id } } }));
            refresh();
          }}
        />
      )}
    </Card>
  );
}
