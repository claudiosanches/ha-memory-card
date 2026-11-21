import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useTranslation } from 'react-i18next';
import { api } from '../../api/client';
import { call, errorMessage } from '../../api/errors';
import { queryKeys } from '../../api/queries';
import type { PlayStatus } from '../../api/types';
import { notifyError, notifySuccess } from '../../components/notify';

type Change = { id: number; status: PlayStatus | null };

/** Sets a game's play status. `quiet` skips the success notice, for quick runs of changes. */
export function useSetPlayStatus({ quiet = false } = {}) {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, status }: Change) =>
      call(api.PUT('/api/games/{id}/play-status', { params: { path: { id } }, body: { play_status: status } })),
    onSuccess: (game, { status }) => {
      queryClient.setQueryData(queryKeys.game(game.id), game);
      void queryClient.invalidateQueries({ queryKey: queryKeys.games });
      if (!quiet) {
        notifySuccess(
          t('backlog.moved', { title: game.title, status: status ? t(`playStatus.${status}`) : t('common.notSet') }),
        );
      }
    },
    onError: (error) => notifyError(errorMessage(t, error)),
  });
}
