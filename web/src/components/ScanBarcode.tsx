import { Alert, Anchor, Badge, Button, Group, Loader, Modal, Stack, Text, TextInput } from '@mantine/core';
import { IconBarcode } from '@tabler/icons-react';
import { Link } from '@tanstack/react-router';
import { useEffect, useRef, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../api/client';
import { call, errorMessage } from '../api/errors';
import { useCanManage, usePlatforms } from '../api/queries';
import type { BarcodeLookup, CopyStatus } from '../api/types';
import { CopyForm } from '../pages/game/forms';
import { createGame, PickGameModal, type Target } from './PickGame';

type Step =
  | { kind: 'reading' }
  | { kind: 'code'; code: string; error?: string }
  | { kind: 'result'; lookup: BarcodeLookup }
  | { kind: 'pick'; lookup: BarcodeLookup; title: string }
  | { kind: 'copy'; lookup: BarcodeLookup; target: Target }
  | null;

/** Scans a box's barcode from a photo, says whether you own it, and adds the copy when you do not. */
export function ScanBarcodeButton({ status }: { status: CopyStatus }) {
  const { t } = useTranslation();
  const input = useRef<HTMLInputElement>(null);
  const [step, setStep] = useState<Step>(null);
  const [busy, setBusy] = useState(false);
  const close = () => setStep(null);

  const lookUp = async (code: string) => {
    setBusy(true);
    try {
      const lookup = await call(api.GET('/api/barcodes/{code}', { params: { path: { code } } }));
      setStep({ kind: 'result', lookup });
    } catch (error) {
      setStep({ kind: 'code', code, error: errorMessage(t, error) });
    } finally {
      setBusy(false);
    }
  };

  const read = async (file: File) => {
    setStep({ kind: 'reading' });
    try {
      // Loaded on first use: the decoder is large and most visits never scan.
      const { decodeBarcode } = await import('./barcode');
      const code = await decodeBarcode(file);
      if (code) await lookUp(code);
      else setStep({ kind: 'code', code: '', error: t('scan.notFound') });
    } catch {
      setStep({ kind: 'code', code: '', error: t('scan.notFound') });
    }
  };

  return (
    <>
      <Button variant="default" leftSection={<IconBarcode size={16} />} onClick={() => input.current?.click()}>
        {t('scan.button')}
      </Button>
      <input
        ref={input}
        type="file"
        accept="image/*"
        capture="environment"
        hidden
        data-testid="scan-input"
        onChange={(event) => {
          const file = event.currentTarget.files?.[0];
          event.currentTarget.value = '';
          if (file) void read(file);
        }}
      />
      {step && step.kind !== 'pick' && step.kind !== 'copy' && (
        <Modal opened onClose={close} title={t('scan.title')} centered>
          {step.kind === 'reading' ? (
            <Group gap="sm">
              <Loader size="sm" />
              <Text size="sm">{t('scan.reading')}</Text>
            </Group>
          ) : step.kind === 'code' ? (
            <CodeEntry step={step} busy={busy} onSubmit={lookUp} onRetry={() => input.current?.click()} />
          ) : (
            <Result
              lookup={step.lookup}
              onNext={(title) => setStep({ kind: 'pick', lookup: step.lookup, title })}
              onClose={close}
            />
          )}
        </Modal>
      )}
      {step?.kind === 'pick' && (
        <PickGameModal
          title={t('scan.whichGame')}
          submitLabel={t('wishlist.next')}
          initialTitle={step.title}
          onClose={close}
          onNext={(target) => setStep({ kind: 'copy', lookup: step.lookup, target })}
        />
      )}
      {step?.kind === 'copy' && <ScannedCopy lookup={step.lookup} target={step.target} status={status} onClose={close} />}
    </>
  );
}

function CodeEntry({
  step,
  busy,
  onSubmit,
  onRetry,
}: {
  step: { code: string; error?: string };
  busy: boolean;
  onSubmit: (code: string) => void;
  onRetry: () => void;
}) {
  const { t } = useTranslation();
  const [code, setCode] = useState(step.code);
  return (
    <form
      onSubmit={(event) => {
        event.preventDefault();
        if (code.trim()) onSubmit(code.trim());
      }}
    >
      <Stack>
        {step.error && <Alert color="yellow">{step.error}</Alert>}
        <TextInput
          label={t('scan.code')}
          inputMode="numeric"
          data-autofocus
          value={code}
          onChange={(event) => setCode(event.currentTarget.value)}
        />
        <Group justify="flex-end">
          <Button variant="default" onClick={onRetry}>
            {t('scan.again')}
          </Button>
          <Button type="submit" loading={busy}>
            {t('scan.lookUp')}
          </Button>
        </Group>
      </Stack>
    </form>
  );
}

function Result({ lookup, onNext, onClose }: { lookup: BarcodeLookup; onNext: (title: string) => void; onClose: () => void }) {
  const { t } = useTranslation();
  const canManage = useCanManage();
  const platforms = usePlatforms();
  const platformName = new Map((platforms.data ?? []).map((p) => [p.id, p.name]));

  if (lookup.lookup === 'in_collection') {
    return (
      <Stack data-testid="scan-owned">
        <Text size="sm">{t('scan.owned', { barcode: lookup.barcode })}</Text>
        {lookup.copies.map((copy) => (
          <Group key={copy.copy_id} gap="xs">
            <Anchor
              renderRoot={(props) => <Link to="/games/$gameId" params={{ gameId: String(copy.game_id) }} {...props} />}
              onClick={onClose}
            >
              {copy.title}
            </Anchor>
            <Text size="sm" c="dimmed">
              {[platformName.get(copy.platform_id), t(`region.${copy.region}`)].join(t('common.separator'))}
            </Text>
            {copy.status === 'wanted' && (
              <Badge size="xs" variant="light" color="pink">
                {t('copyStatus.badge')}
              </Badge>
            )}
          </Group>
        ))}
      </Stack>
    );
  }

  const product = lookup.product;
  return (
    <Stack data-testid="scan-result">
      <Text size="sm" c="dimmed">
        {t('scan.read', { barcode: lookup.barcode })}
      </Text>
      {product ? (
        <Text fw={600}>
          {product.title}
          {product.brand && (
            <Text span c="dimmed" fw={400}>
              {' '}
              ({product.brand})
            </Text>
          )}
        </Text>
      ) : (
        <Text size="sm">
          {lookup.lookup === 'offline' ? t('scan.offline') : lookup.lookup === 'failed' ? t('scan.failed') : t('scan.unknown')}
        </Text>
      )}
      {canManage && (
        <Group justify="flex-end">
          <Button
            onClick={async () => {
              if (!product) return onNext('');
              const { gameTitle, platformInTitle } = await import('./barcode');
              const platform = platformInTitle(product.title, platforms.data ?? []);
              onNext(gameTitle(product.title, platform) || product.title);
            }}
          >
            {t('scan.addIt')}
          </Button>
        </Group>
      )}
    </Stack>
  );
}

/** The copy form for a scanned box, with the barcode and the platform and region it suggests. */
function ScannedCopy({ lookup, target, status, onClose }: { lookup: BarcodeLookup; target: Target; status: CopyStatus; onClose: () => void }) {
  const { t } = useTranslation();
  const platforms = usePlatforms();
  const created = useRef<number | null>(null);
  const [guess, setGuess] = useState<{ platform_id?: number; region?: BarcodeLookup['copies'][number]['region'] | null } | null>(null);

  useEffect(() => {
    if (!platforms.data) return;
    let current = true;
    void import('./barcode').then(({ platformInTitle, regionFromBarcode }) => {
      const active = (platforms.data ?? []).filter((p) => p.active);
      const platform = lookup.product ? platformInTitle(lookup.product.title, active) : null;
      if (current) setGuess({ platform_id: platform?.id, region: regionFromBarcode(lookup.barcode) });
    });
    return () => {
      current = false;
    };
  }, [platforms.data, lookup]);

  if (guess === null) return null;

  const resolveGame = async () => {
    if (target.kind === 'existing') return target.id;
    created.current ??= await createGame(target, t);
    return created.current;
  };
  return (
    <CopyForm
      resolveGame={resolveGame}
      status={status}
      scanned={{ barcode: lookup.barcode, ...guess }}
      title={t('scan.addTitle', { title: target.title })}
      onClose={onClose}
    />
  );
}
