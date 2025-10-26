import { invoke } from '@tauri-apps/api/core';
import type { VideoMeta } from './types';

export async function probeVideo(path: string): Promise<VideoMeta> {
  const meta = await invoke<VideoMeta>('probe_video', { path });
  return meta;
}
