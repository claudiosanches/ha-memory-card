import { Button, Stack, Text, Title } from '@mantine/core';
import { IconCheck } from '@tabler/icons-react';
import { Link } from '@tanstack/react-router';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useCanManage, useDevices, useStores } from '../../api/queries';
import type { Device } from '../../api/types';
import { formatMoney } from '../../components/money';
import classes from '../manage/ManageList.module.css';
import { HardwareForm } from './HardwareForm';

/** Hardware on the wishlist, with Got it for managers. */
export function WishlistHardware() {
  const { t, i18n } = useTranslation();
  const canManage = useCanManage();
  const devices = useDevices();
  const stores = useStores();
  const [gotIt, setGotIt] = useState<Device | null>(null);
  const wanted = (devices.data ?? []).filter((d) => d.status === 'wanted');
  if (wanted.length === 0) return null;

  return (
    <Stack gap="sm" data-testid="wishlist-hardware">
      <Title order={3}>
        <Link to="/hardware">{t('nav.hardware')}</Link>
      </Title>
      <ul className={classes.grid}>
        {wanted.map((device) => {
          const store = device.store_id ? stores.data?.find((s) => s.id === device.store_id)?.name : null;
          const line = [
            t(`deviceKind.${device.kind}`),
            device.model,
            store && t('wishlist.store', { store }),
            device.price && device.currency && formatMoney(device.price, device.currency, i18n.language),
          ].filter(Boolean);
          return (
            <li key={device.id} className={classes.card}>
              <div className={classes.top}>
                <div className={classes.main}>
                  <Text className={classes.name}>{device.name}</Text>
                  <Text size="sm" c="dimmed">
                    {line.join(t('common.separator'))}
                  </Text>
                </div>
                {canManage && (
                  <Button
                    size="xs"
                    variant="light"
                    color="green"
                    leftSection={<IconCheck size={16} />}
                    onClick={() => setGotIt(device)}
                  >
                    {t('game.gotIt')}
                  </Button>
                )}
              </div>
            </li>
          );
        })}
      </ul>
      {gotIt && <HardwareForm device={gotIt} status="owned" onClose={() => setGotIt(null)} />}
    </Stack>
  );
}
