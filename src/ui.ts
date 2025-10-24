// Drag & drop only + badges + controls

import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

const drop = document.getElementById('drop') as HTMLElement | null;
const hint = document.getElementById('hint') as HTMLElement | null;
const badges = document.getElementById('badges') as HTMLElement | null;
const btnAdd = document.getElementById('btnAdd') as HTMLButtonElement | null;
const fileInput = document.getElementById('fileInput') as HTMLInputElement | null;
const btnInstall = document.getElementById('btnInstall') as HTMLButtonElement | null;
const btnExit = document.getElementById('btnExit') as HTMLButtonElement | null;
const trimBlock = document.getElementById('trimBlock') as HTMLElement | null;
const trimStart = document.getElementById('trimStart') as HTMLInputElement | null;
const trimEnd = document.getElementById('trimEnd') as HTMLInputElement | null;
const size1080 = document.getElementById('size1080') as HTMLInputElement | null;
const size1440 = document.getElementById('size1440') as HTMLInputElement | null;
const size2160 = document.getElementById('size2160') as HTMLInputElement | null;
// Target browser checkboxes
const tOpera = document.getElementById('tOpera') as HTMLInputElement | null;
const tGX = document.getElementById('tGX') as HTMLInputElement | null;

const setActive = (on: boolean) => {
  if (!drop) return;
  drop.classList.toggle('active', on);
};

let currentPath: string | null = null;
const showPicked = (path: string) => {
  if (hint) {
    const name = path.split('/').pop() || path;
    hint.textContent = `Selected: ${name}`;
  }
};

const renderBadges = (path: string, meta: any) => {
  if (!badges) return;
  const name = path.split('/').pop() || path;
  const duration = Number(meta?.duration ?? 0);
  const w = meta?.width ?? '?';
  const h = meta?.height ?? '?';
  const fps = meta?.fps ? Number(meta.fps).toFixed(2) : '?';
  const vfr = meta?.vfr ? 'VFR' : '';
  const items: string[] = [
    `<span class="badge">${name}</span>`,
    `<span class="badge">${duration.toFixed(2)}s</span>`,
    `<span class="badge">${w}×${h}</span>`,
    `<span class="badge">${fps} FPS</span>`
  ];
  if (vfr) items.push(`<span class="badge warn">${vfr}</span>`);
  badges.innerHTML = items.join('');
};

const handleFilePath = async (path: string) => {
  currentPath = path;
  showPicked(path);
  if (btnInstall) btnInstall.disabled = true;
  if (badges) badges.innerHTML = '';
  try {
    const meta = await invoke<any>('probe_video', { path });
    renderBadges(path, meta);
    if (btnInstall) btnInstall.disabled = false;
    const dur = Number(meta?.duration ?? 0);
    if (trimStart && trimEnd && trimBlock) {
      if (dur > 60) {
        trimBlock.classList.add('visible');
        trimStart.value = '0';
        trimEnd.value = String(Math.min(60, Math.max(5, Math.floor(dur * 10) / 10)));
      } else {
        trimBlock.classList.remove('visible');
        trimStart.value = '0';
        trimEnd.value = String(Math.floor(dur * 10) / 10);
      }
    }
  } catch (err) {
    if (badges) badges.innerHTML = `<span class="badge warn">probe failed: ${(err as any)?.toString?.() ?? err}</span>`;
  }
};

// DOM drag events (visual only)
if (drop) {
  ;['dragenter', 'dragover'].forEach((ev) => {
    drop.addEventListener(ev, (e) => {
      e.preventDefault();
      e.stopPropagation();
      setActive(true);
    });
  });
  ;['dragleave', 'drop'].forEach((ev) => {
    drop.addEventListener(ev, (e) => {
      e.preventDefault();
      e.stopPropagation();
      setActive(false);
    });
  });

  // Fallback: browser DataTransfer (dev in browser)
  drop.addEventListener('drop', (e) => {
    const dt = (e as DragEvent).dataTransfer;
    const f = dt?.files?.[0];
    if (f && (f as any).path) handleFilePath(((f as any).path) as string);
  });
}

// Prevent browser default on the whole document to avoid navigating to file
;['dragenter','dragover','dragleave','drop'].forEach((ev) => {
  window.addEventListener(ev, (e) => {
    e.preventDefault();
  });
});

// Tauri native drag/drop events (v2): use appWindow.onDragDropEvent
try {
  const appWindow = getCurrentWindow();
  appWindow.onDragDropEvent((ev) => {
    const t = (ev as any)?.payload?.type;
    if (t === 'enter' || t === 'over') setActive(true);
    if (t === 'leave') setActive(false);
    if (t === 'drop') {
      setActive(false);
      const p = (ev as any)?.payload?.paths?.[0];
      if (p) handleFilePath(p as string);
    }
  });
} catch (err) {
  console.warn('onDragDropEvent not available', err);
}

// Controls
const appWindow = getCurrentWindow();
btnExit?.addEventListener('click', () => appWindow.close());
btnInstall?.addEventListener('click', async () => {
  try {
    if (!currentPath) return;
    const sizes: number[] = [];
    if (size1080?.checked) sizes.push(1080);
    if (size1440?.checked) sizes.push(1440);
    if (size2160?.checked) sizes.push(2160);
    // sizes optional for new packer; keep UI validation minimal
    const start = parseFloat(trimStart?.value || '0') || 0;
    const end = parseFloat(trimEnd?.value || '0') || 0;
    if (hint) hint.textContent = 'Exporting…';
    btnInstall!.disabled = true;
    const name = (currentPath.split('/').pop() || 'wallpaper').replace(/\.[^.]+$/, '');
    const targets: string[] = collectTargets();
    const zipPath = await invoke<string>('export_package', {
      req: { path: currentPath, start, end, sizes, name, targets },
    });
    if (hint) hint.textContent = `Done: ${zipPath}`;
  } catch (e) {
    if (hint) hint.textContent = `Export failed: ${e}`;
  } finally {
    btnInstall!.disabled = false;
  }
});

// Add button
let opening = false;
btnAdd?.addEventListener('click', async (e) => {
  e.stopPropagation();
  if (opening) return;
  opening = true;
  try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [
        { name: 'Video', extensions: ['mp4', 'mov', 'webm', 'gif'] },
        { name: 'All files', extensions: ['*'] },
      ],
    });
    if (typeof selected === 'string') {
      await handleFilePath(selected);
      return;
    }
    fileInput?.click();
  } catch (_) {
    fileInput?.click();
  } finally {
    opening = false;
  }
});

fileInput?.addEventListener('change', async () => {
  const f = fileInput.files?.[0];
  if (f && (f as any).path) {
    currentPath = (f as any).path as string;
    await handleFilePath(currentPath);
  }
  fileInput.value = '';
});

// Enforce exclusive selection among 1080/1440/2160 checkboxes
function sizeInputs(): HTMLInputElement[] {
  return [size1080, size1440, size2160].filter(Boolean) as HTMLInputElement[];
}

function uncheckOthers(active: HTMLInputElement) {
  for (const el of sizeInputs()) {
    if (el !== active) el.checked = false;
  }
}

for (const el of sizeInputs()) {
  el.addEventListener('change', () => {
    if (el.checked) uncheckOthers(el);
  });
}

// Targets detection and persistence
type TargetsSave = { opera?: boolean; gx?: boolean };
function saveTargets() {
  const data: TargetsSave = {
    opera: !!tOpera?.checked,
    gx: !!tGX?.checked,
  };
  try { localStorage.setItem('operaLW.targets', JSON.stringify(data)); } catch {}
}

function loadTargets(): TargetsSave | null {
  try { const s = localStorage.getItem('operaLW.targets'); return s ? JSON.parse(s) : null; } catch { return null; }
}

function collectTargets(): string[] {
  const res: string[] = [];
  if (tOpera?.checked && tOpera.dataset.path) res.push(tOpera.dataset.path);
  if (tGX?.checked && tGX.dataset.path) res.push(tGX.dataset.path);
  saveTargets();
  return res;
}

async function detectTargets() {
  try {
    const paths = await invoke<string[]>('detect_theme_targets');
    const operaPath = paths.find((p) => p.includes('com.operasoftware.Opera/themes')) || '';
    const gxPath = paths.find((p) => p.includes('com.operasoftware.OperaGX/themes')) || '';
    if (tOpera) { tOpera.dataset.path = operaPath; tOpera.checked = !!operaPath; }
    if (tGX) { tGX.dataset.path = gxPath; tGX.checked = !!gxPath; }
    const saved = loadTargets();
    if (saved) {
      if (tOpera) tOpera.checked = saved.opera ?? tOpera.checked;
      if (tGX) tGX.checked = saved.gx ?? tGX.checked;
    }
  } catch (e) {
    console.warn('detect targets failed', e);
  }
}

detectTargets();
