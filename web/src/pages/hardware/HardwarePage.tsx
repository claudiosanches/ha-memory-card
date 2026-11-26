import { ActionIcon, Badge, Button, Group, Loader, SegmentedControl, Select, Stack, Text } from '@mantine/core';
import { IconCamera, IconDeviceGamepad, IconDevices, IconPencil, IconPlus, IconTrash } from '@tabler/icons-react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useMemo, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys, useCanManage, useDevices, usePlatforms, useStores } from '../../api/queries';
import { type CopyStatus, type Device, type DeviceKind, DEVICE_KINDS, holdsGames } from '../../api/types';
import { ConfirmModal } from '../../components/ConfirmModal';
import { formatMoney } from '../../components/money';
import { notifyError, notifySuccess } from '../../components/notify';
import { PageHeader } from '../../components/PageHeader';
import { useFormatDate } from '../../hooks/useFormatDate';
import copyClasses from '../game/Copies.module.css';
import { PhotoViewer } from '../game/PhotoViewer';
import classes from '../manage/ManageList.module.css';
import { HardwareForm } from './HardwareForm';

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

  const shown = (devices.data ?? []).filter(
    (d) => (show === 'all' || d.status === show) && (!kind || d.kind === kind),
  );

  return (
    <Stack gap="section">
      <PageHeader title={t('nav.hardware')} description={t('hardware.intro')}>
        {canManage && (
          <Button leftSection={<IconPlus size={16} />} onClick={() => setEditing({})}>
            {t('hardware.add')}
          </Button>
        )}
      </PageHeader>
      <Group gap="sm" align="flex-end">
        <SegmentedControl
          value={show}
          onChange={(value) => setShow(value as Show)}
          data={[
            { value: 'owned', label: t('hardware.showOwned') },
            { value: 'wanted', label: t('nav.wishlist') },
            { value: 'all', label: t('hardware.showAll') },
          ]}
        />
        <Select
          aria-label={t('devices.kind')}
          placeholder={t('hardware.allKinds')}
          data={DEVICE_KINDS.map((k) => ({ value: k, label: t(`deviceKind.${k}`) }))}
          value={kind}
          onChange={(value) => setKind(value as DeviceKind | null)}
          clearable
          w={200}
        />
      </Group>

      {devices.isPending || platforms.isPending ? (
        <Loader size="sm" />
      ) : devices.isError ? (
        <Text c="red">{errorMessage(t, devices.error)}</Text>
      ) : shown.length === 0 ? (
        <div className={classes.empty}>{t('hardware.empty')}</div>
      ) : (
        <ul className={classes.grid}>
          {shown.map((device) => (
            <HardwareCard
              key={device.id}
              device={device}
              canManage={canManage}
              onEdit={(status) => setEditing({ device: status ? { ...device, status } : device })}
              onDelete={() => setDeleting(device)}
            />
          ))}
        </ul>
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
  const platformName = useMemo(() => new Map((platforms.data ?? []).map((p) => [p.id, p.abbreviation ?? p.name])), [platforms.data]);
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
  const Icon = device.kind === 'controller' ? IconDeviceGamepad : IconDevices;

  return (
    <li className={classes.card} data-testid={`device-${device.name}`}>
      <div className={classes.top}>
        <Icon size={20} stroke={1.5} aria-hidden style={{ flexShrink: 0, marginTop: 2 }} />
        <div className={classes.main}>
          <Group gap="xs">
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
          </Group>
          <Text size="sm" c="dimmed">
            {[t(`deviceKind.${device.kind}`), device.model].filter(Boolean).join(t('common.separator'))}
          </Text>
          {details.length > 0 && (
            <Text size="xs" c="dimmed">
              {details.join(t('common.separator'))}
            </Text>
          )}
          {prices.length > 0 && <Text size="xs">{prices.join(t('common.separator'))}</Text>}
          {device.notes && (
            <Text size="xs" c="dimmed" className={classes.notes}>
              {device.notes}
            </Text>
          )}
        </div>
        {canManage && (
          <Group gap={4} wrap="nowrap" className={classes.actions}>
            {wanted && (
              <Button size="compact-xs" variant="light" onClick={() => onEdit('owned')}>
                {t('game.gotIt')}
              </Button>
            )}
            <ActionIcon
              variant="subtle"
              aria-label={t('photos.add')}
              loading={upload.isPending}
              onClick={() => input.current?.click()}
            >
              <IconCamera size={16} />
            </ActionIcon>
            <ActionIcon variant="subtle" aria-label={t('common.edit')} onClick={() => onEdit()}>
              <IconPencil size={16} />
            </ActionIcon>
            <ActionIcon variant="subtle" color="red" aria-label={t('common.delete')} onClick={onDelete}>
              <IconTrash size={16} />
            </ActionIcon>
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
      {device.photos.length > 0 && (
        <div className={copyClasses.photos}>
          {device.photos.map((photo, i) => (
            <button
              key={photo.id}
              type="button"
              className={copyClasses.thumb}
              aria-label={t('photos.open', { number: i + 1 })}
              onClick={() => setViewing(i)}
            >
              <img src={`covers/${photo.path}`} alt="" loading="lazy" decoding="async" />
            </button>
          ))}
        </div>
      )}
      <div className={classes.footer}>
        <Group gap={4}>
          {(holdsGames(device.kind) ? device.platform_ids : device.platform_id ? [device.platform_id] : []).map((id) => (
            <Badge key={id} variant="outline" size="sm" tt="none">
              {platformName.get(id)}
            </Badge>
          ))}
        </Group>
        {holdsGames(device.kind) && (
          <Text size="xs" c="dimmed" className={classes.count}>
            {t('common.copyCount', { count: device.copy_count })}
          </Text>
        )}
      </div>
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
    </li>
  );
}
