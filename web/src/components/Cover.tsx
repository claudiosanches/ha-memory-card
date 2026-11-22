import { IconDeviceGamepad2 } from '@tabler/icons-react';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

/** Game cover at a 3:4 ratio, or a placeholder when there is none. */
export function Cover({
  path,
  title,
  width = '100%',
  onError,
}: {
  path?: string | null;
  title: string;
  width?: number | string;
  onError?: () => void;
}) {
  const { t } = useTranslation();
  const [failedPath, setFailedPath] = useState<string | null>(null);
  // Relative, so it resolves under the ingress path.
  return (
    <div className="mc-cover" style={{ width }}>
      {path && path !== failedPath ? (
        <img
          src={`covers/${path}`}
          alt={t('game.cover', { title })}
          loading="lazy"
          decoding="async"
          onError={() => {
            setFailedPath(path);
            onError?.();
          }}
        />
      ) : (
        <IconDeviceGamepad2 aria-hidden="true" stroke={1.2} />
      )}
    </div>
  );
}
