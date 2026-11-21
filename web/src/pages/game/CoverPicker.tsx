import { Loader, Menu, UnstyledButton } from '@mantine/core';
import { IconPhotoEdit, IconTrash, IconUpload } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { Cover } from '../../components/Cover';
import { notifyError } from '../../components/notify';
import classes from './CoverPicker.module.css';
import { useGameSaved } from './forms';

export type CoverTarget = { kind: 'game' | 'release'; id: number };

const ACCEPT = 'image/jpeg,image/png,image/webp';

// The server reads the image type from the bytes, so the body goes as is.
const asImage = (file: File) => ({
  body: file as unknown as string,
  bodySerializer: (body: unknown) => body as BodyInit,
  headers: { 'Content-Type': 'application/octet-stream' },
});

function uploadCover(target: CoverTarget, file: File) {
  const params = { path: { id: target.id } };
  return target.kind === 'game'
    ? call(api.PUT('/api/games/{id}/cover', { params, ...asImage(file) }))
    : call(api.PUT('/api/releases/{id}/cover', { params, ...asImage(file) }));
}

function removeCover(target: CoverTarget) {
  const params = { path: { id: target.id } };
  return target.kind === 'game'
    ? call(api.DELETE('/api/games/{id}/cover', { params }))
    : call(api.DELETE('/api/releases/{id}/cover', { params }));
}

/** A cover that opens a menu to upload a replacement or remove an upload. */
export function CoverPicker({
  target,
  path,
  uploaded,
  title,
  width,
}: {
  target: CoverTarget;
  path: string | null | undefined;
  uploaded: boolean;
  title: string;
  width: number;
}) {
  const { t } = useTranslation();
  const saved = useGameSaved();
  const input = useRef<HTMLInputElement>(null);

  const upload = useMutation({
    mutationFn: (file: File) => uploadCover(target, file),
    onSuccess: (game) => saved(game, t('cover.uploaded')),
    onError: (error) => notifyError(errorMessage(t, error)),
  });
  const remove = useMutation({
    mutationFn: () => removeCover(target),
    onSuccess: (game) => saved(game, t('cover.removed')),
    onError: (error) => notifyError(errorMessage(t, error)),
  });
  const busy = upload.isPending || remove.isPending;
  const small = width < 64;

  return (
    <>
      <Menu position="bottom-start" shadow="md">
        <Menu.Target>
          <UnstyledButton
            className={`${classes.button} ${small ? classes.small : ''}`}
            aria-label={t(target.kind === 'game' ? 'cover.changeGame' : 'cover.changeRelease')}
            disabled={busy}
          >
            <Cover path={path} title={title} width={width} />
            <span className={classes.hint} aria-hidden>
              {busy ? <Loader size={small ? 10 : 12} /> : <IconPhotoEdit size={small ? 12 : 14} />}
            </span>
          </UnstyledButton>
        </Menu.Target>
        <Menu.Dropdown>
          <Menu.Item leftSection={<IconUpload size={14} />} onClick={() => input.current?.click()}>
            {t('cover.upload')}
          </Menu.Item>
          {uploaded && (
            <Menu.Item color="red" leftSection={<IconTrash size={14} />} onClick={() => remove.mutate()}>
              {t('cover.remove')}
            </Menu.Item>
          )}
        </Menu.Dropdown>
      </Menu>
      <input
        ref={input}
        type="file"
        accept={ACCEPT}
        hidden
        data-testid={`cover-file-${target.kind}-${target.id}`}
        onChange={(event) => {
          const file = event.currentTarget.files?.[0];
          event.currentTarget.value = '';
          if (file) upload.mutate(file);
        }}
      />
    </>
  );
}
