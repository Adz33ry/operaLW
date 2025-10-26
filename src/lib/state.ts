import type { VideoMeta } from './types';

export type AppState = {
  meta: VideoMeta | null;
  ready: boolean;
};

export const state: AppState = {
  meta: null,
  ready: false,
};

