import type { MonitorInfo } from "@/libs/monitors";

/** The maintainer's Windows desk: one ultrawide */
export const single: MonitorInfo[] = [{ id: "\\\\.\\DISPLAY1", x: 0, y: 0, width: 2560, height: 1080, scale: 1, primary: true }];

/** An ultrawide with a 1080p screen to its left, the primary on the right */
export const dual: MonitorInfo[] = [
  { id: "\\\\.\\DISPLAY1", x: 0, y: 0, width: 2560, height: 1080, scale: 1, primary: true },
  { id: "\\\\.\\DISPLAY2", x: -1920, y: 0, width: 1920, height: 1080, scale: 1, primary: false },
];

/** A laptop at 150 % under and left of a 4K monitor, with a portrait screen to the right */
export const triple: MonitorInfo[] = [
  { id: "\\\\.\\DISPLAY1", x: 0, y: 0, width: 3840, height: 2160, scale: 1.5, primary: true },
  { id: "\\\\.\\DISPLAY2", x: -2880, y: 1080, width: 2880, height: 1800, scale: 2, primary: false },
  { id: "\\\\.\\DISPLAY3", x: 3840, y: -400, width: 1440, height: 2560, scale: 1, primary: false },
];

/** Two monitors stacked, the primary underneath */
export const stacked: MonitorInfo[] = [
  { id: "\\\\.\\DISPLAY1", x: 0, y: 0, width: 1920, height: 1080, scale: 1, primary: true },
  { id: "\\\\.\\DISPLAY2", x: 0, y: -1440, width: 2560, height: 1440, scale: 1, primary: false },
];
