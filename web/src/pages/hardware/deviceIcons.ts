import {
  IconBrandAppleArcade,
  IconDeviceGamepad2,
  IconDevicesPc,
  IconPuzzle,
  IconRobot,
} from '@tabler/icons-react';
import { createElement, type ComponentType } from 'react';
import type { DeviceKind } from '../../api/types';

export type DeviceIconProps = { size?: number; stroke?: number; 'aria-hidden'?: boolean };
export type DeviceIcon = ComponentType<DeviceIconProps>;

/** Classic pocket handheld silhouette drawn on Tabler's 24x24 outline grid. */
function IconHandheld({ size = 24, stroke = 2, ...rest }: DeviceIconProps) {
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
    createElement('rect', { x: 4, y: 3, width: 16, height: 18, rx: 2 }),
    createElement('rect', { x: 7, y: 6, width: 10, height: 5.5, rx: 1 }),
    createElement('path', { d: 'M7.5 16h3m-1.5 -1.5v3' }),
    createElement('path', { d: 'M14 17l0 .01' }),
    createElement('path', { d: 'M16.5 15l0 .01' }),
  );
}

/** Navigation icon for the Hardware section. */
export const HARDWARE_NAV_ICON: DeviceIcon = IconBrandAppleArcade;

/** Icon representing each hardware kind across cards and group headings. */
export const DEVICE_KIND_ICON: Record<DeviceKind, DeviceIcon> = {
  console: IconBrandAppleArcade,
  handheld: IconHandheld,
  pc: IconDevicesPc,
  controller: IconDeviceGamepad2,
  accessory: IconPuzzle,
  toy: IconRobot,
};
