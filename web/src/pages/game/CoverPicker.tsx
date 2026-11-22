import { Loader, Menu, Modal, UnstyledButton } from '@mantine/core';
import { IconPhotoEdit, IconTrash, IconUpload } from '@tabler/icons-react';
import { useMutation } from '@tanstack/react-query';
import { useRef, useState } from 'react';
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

/**
 * A cover that opens full size when clicked, with a corner button whose menu
 * uploads a replacement or removes an upload. A missing cover opens the menu.
 */
export function CoverPicker({
  target,
  path,
  uploaded,
  title,
  width,
  readOnly = false,
}: {
  target: CoverTarget;
  path: string | null | undefined;
  uploaded: boolean;
  title: string;
  width: number;
  readOnly?: boolean;
}) {
  const { t } = useTranslation();
  const saved = useGameSaved();
  const input = useRef<HTMLInputElement>(null);
  const [viewing, setViewing] = useState(false);
  const [menu, setMenu] = useState(false);
  const [broken, setBroken] = useState<string | null>(null);
  const viewable = Boolean(path) && path !== broken;

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
    <div className={`${classes.picker} ${small ? classes.small : ''}`}>
      {/* One element in every state, so the image does not load again. With nothing
          to view, the placeholder opens the menu; the corner button is the accessible way in. */}
      <UnstyledButton
        className={classes.open}
        {...(viewable
          ? { 'aria-label': t('cover.open', { title }), onClick: () => setViewing(true) }
          : { tabIndex: -1, 'aria-hidden': true, 'data-placeholder': true, disabled: readOnly, onClick: () => setMenu(true) })}
      >
        <Cover path={path} title={title} width={width} onError={() => setBroken(path ?? null)} />
      </UnstyledButton>
      {!readOnly && (
        <Menu position="bottom-start" shadow="md" opened={menu} onChange={setMenu}>
          <Menu.Target>
            <UnstyledButton
              className={classes.change}
              aria-label={t(target.kind === 'game' ? 'cover.changeGame' : 'cover.changeRelease')}
              disabled={busy}
            >
              {busy ? <Loader size={small ? 10 : 12} /> : <IconPhotoEdit size={small ? 12 : 14} />}
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
      )}
      {viewing && viewable && (
        <Modal opened onClose={() => setViewing(false)} title={title} size="auto" centered>
          <img src={`covers/${path}`} alt={t('game.cover', { title })} className={classes.full} />
        </Modal>
      )}
      {!readOnly && (
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
      )}
    </div>
  );
}
