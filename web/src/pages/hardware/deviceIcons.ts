import { IconDeviceGamepad2, IconDevicesPc, IconPuzzle, IconRobot } from '@tabler/icons-react';
import { createElement, type ComponentType, type ReactNode } from 'react';
import type { DeviceKind } from '../../api/types';

export type DeviceIconProps = { size?: number; stroke?: number; 'aria-hidden'?: boolean };
export type DeviceIcon = ComponentType<DeviceIconProps>;

/** An outline icon on Tabler's 24x24 grid, drawn like Tabler's own. */
function outline({ size = 24, stroke = 2, ...rest }: DeviceIconProps, ...shapes: ReactNode[]) {
  return createElement(
    'svg',
    {
      xmlns: 'http://www.w3.org/2000/svg',
      width: size,
      height: size,
      viewBox: '0 0 24 24',
      fill: 'none',
      stroke: 'currentColor',
      strokeWidth: stroke,
      strokeLinecap: 'round',
      strokeLinejoin: 'round',
      ...rest,
    },
    ...shapes,
  );
}

/** Classic pocket handheld silhouette. */
function IconHandheld(props: DeviceIconProps) {
  return outline(
    props,
    createElement('rect', { key: 'body', x: 4, y: 3, width: 16, height: 18, rx: 2 }),
    createElement('rect', { key: 'screen', x: 7, y: 6, width: 10, height: 5.5, rx: 1 }),
    createElement('path', { key: 'pad', d: 'M7.5 16h3m-1.5 -1.5v3' }),
    createElement('path', { key: 'a', d: 'M14 17l0 .01' }),
    createElement('path', { key: 'b', d: 'M16.5 15l0 .01' }),
  );
}

/** A home console: a disc slot and power light above two controller ports. */
function IconConsole(props: DeviceIconProps) {
  return outline(
    props,
    createElement('rect', { key: 'body', x: 3, y: 5, width: 18, height: 14, rx: 2 }),
    createElement('path', { key: 'panel', d: 'M3 14h18' }),
    createElement('path', { key: 'slot', d: 'M6.5 9.5h8' }),
    createElement('path', { key: 'light', d: 'M17.5 9.5l0 .01' }),
    createElement('path', { key: 'ports', d: 'M7 16.5h2m6 0h2' }),
  );
}

/** Navigation icon for the Hardware section. */
export const HARDWARE_NAV_ICON: DeviceIcon = IconConsole;

/** Icon representing each hardware kind across cards and group headings. */
export const DEVICE_KIND_ICON: Record<DeviceKind, DeviceIcon> = {
  console: IconConsole,
  handheld: IconHandheld,
  pc: IconDevicesPc,
  controller: IconDeviceGamepad2,
  accessory: IconPuzzle,
  toy: IconRobot,
};
