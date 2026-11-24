import { ColorSwatch, Group, Input } from '@mantine/core';
import { IconCheck } from '@tabler/icons-react';
import { useTranslation } from 'react-i18next';
import { STATUS_COLORS, type StatusColor } from '../api/types';

const LIGHT_SWATCHES = new Set<StatusColor>(['yellow', 'lime']);

/** One of the palette colors play statuses and labels use, as a row of swatches. */
export function ColorPicker({
  label,
  value,
  onChange,
}: {
  label: string;
  value: StatusColor;
  onChange: (color: StatusColor) => void;
}) {
  const { t } = useTranslation();
  return (
    <Input.Wrapper label={label}>
      <Group gap={8} mt={6} role="radiogroup" aria-label={label}>
        {STATUS_COLORS.map((color) => {
          const selected = value === color;
          return (
            <ColorSwatch
              key={color}
              component="button"
              type="button"
              role="radio"
              aria-checked={selected}
              aria-label={t(`playStatuses.colors.${color}`)}
              color={`var(--mantine-color-${color}-filled)`}
              onClick={() => onChange(color)}
              style={{ cursor: 'pointer' }}
            >
              {selected && <IconCheck size={14} color={LIGHT_SWATCHES.has(color) ? 'var(--mantine-color-black)' : 'white'} />}
            </ColorSwatch>
          );
        })}
      </Group>
    </Input.Wrapper>
  );
}
