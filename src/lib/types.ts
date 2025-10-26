export type VideoMeta = {
  path: string;
  duration: number;
  width: number;
  height: number;
  fps: number;
  vfr: boolean;
  codec: string;
};

export type ExportRequest = {
  path: string;
  start: number;
  end: number;
  sizes: number[];
  name: string;
  noCrop?: boolean;
};

export type ExportResponse = {
  zipPath: string;
};

export const SIZES = [1080, 1440, 2160] as const;
export const CRF_VP9 = 30;
export const TRIM_MIN = 5;
export const TRIM_MAX = 60;
