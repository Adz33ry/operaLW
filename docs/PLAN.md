Opera LW — Development Plan (po-krokovo)

Ціль: підготувати локальне відео до формату живих шпалер Opera GX і віддати готовий ZIP (1080p/1440p/2160p, poster.jpg, metadata.json). Один екран 350×650, офлайн, без телеметрії.

⸻

1) Scope & Constraints
	•	Платформа: macOS 12+, Apple Silicon (arm64) only.
	•	Стек: Tauri v2.4.x, Vite + TypeScript, pnpm, Rust 1.80+.
	•	Sidecars: ffmpeg, ffprobe (arm64, у пакеті).
	•	Вхід: відео-файл через drag&drop (MP4/MOV/WEBM/GIF; перевірка лише ffprobe).
	•	Вихід: ZIP з wallpaper_*.webm (VP9, CFR ≤30 fps, без аудіо), poster.jpg, metadata.json.
	•	UX: фіксоване вікно 350×650, EN тексти, Enter=Install; E/Esc/⌘Q=Close (поза експортом).
	•	Безпека: жодних мережевих запитів від додатку (ані dev, ані prod).

⸻

2) Архітектура (огляд)
	•	UI (Vite/TS): drop-zone → бейджі метаданих → (умовно) блок Trim → чекбокси розмірів → кнопка Install.
	•	Tauri/Rust команди:
	•	probe_video(path) -> VideoMeta — обгортка ffprobe.
	•	export_package(req) -> zip_path — trim/scale/crop/fps → VP9 .webm → poster → metadata → zip → reveal+activate Opera GX.
	•	Тимчасові файли: ~/Library/Caches/opera-lw/<uuid>/… (завжди очищати).
	•	Capabilities: core:default, core:event:default, core:window:default, core:window:allow-close, core:path:default, core:webview:default (+devtools у dev).

⸻

3) Дорожня карта (поетапно)

Етап 0 — Ініціалізація репозиторію

Завдання
	•	Створити каркас Vite (vanilla-ts) + Tauri v2.4.x, налаштувати pnpm.
	•	Додати скрипти: dev, tauri:dev, tauri:build.
	•	Створити базові файли: index.html, src/main.ts, src/ui.ts, src/lib/{types.ts,ff.ts,state.ts}.
	•	Налаштувати вікно 350×650, нересайзабельне.
Deliverables: початковий commit, запуск pnpm tauri dev без помилок.
DoD: вікно відкривається, на екрані — порожня drop-зона.

Етап 1 — Sidecars & перевірки середовища

Завдання
	•	Додати ffmpeg/ffprobe у src-tauri/bin/ і до bundle.resources.
	•	Перед запуском команд — -version перевірка і дружнє повідомлення, якщо не знайдено.
Deliverables: робочі виклики sidecar в dev/build.
DoD: probe_video виконує ffprobe -version і повертає OK.

Етап 2 — Команда probe_video

Завдання
	•	Виклик ffprobe та парс JSON: duration, width, height, avg_frame_rate, r_frame_rate, nb_frames, codec_name.
	•	Обчислити fps (із avg_frame_rate), vfr = avg_frame_rate ≠ r_frame_rate || nb_frames missing.
	•	Валідація: якщо нема відео-потоку — помилка “Unsupported format”.
Deliverables: тип VideoMeta, команда probe_video.
DoD: дроп → бейджі заповнюються; для не-відео — відмова.

Етап 3 — UI імпорту

Завдання
	•	Drag&drop з Finder (брати event.payload.paths[0]).
	•	Ніякої фільтрації по розширенню.
	•	Показати бейджі: name, duration, WxH, FPS, VFR (маркер).
	•	Кнопка Install активна лише після успішного probe.
DoD: робочий імпорт 4 форматів; невалідні файли не активують Install.

Етап 4 — Trim/обрізка

Завдання
	•	Показувати блок Trim, якщо duration > 60.
	•	Поля start/end із кроком 0.1 c; примусово в межах 5…60 c; автокорекція виходу за межі.
	•	Прев’ю <video muted loop> із застосуванням меж (візуально).
DoD: 70 s → Trim видно; 12 s → Trim приховано; значення не виходять за 5…60.

Етап 5 — Експорт (VP9) і FPS-політика

Завдання
	•	Політика FPS: TARGET_FPS = min(30, round(input_fps)).
	•	Якщо input_fps > 30 → CFR 30 (фільтр fps=30, -r 30 -vsync cfr).
	•	Якщо input_fps ≤ 30 → не апсемплити (без fps= у фільтрі, -r TARGET_FPS -vsync cfr).
	•	Масштаб/кадрування (без letterbox): scale=-2:H, crop=iw:trunc(iw*9/16).
	•	Розміри на вибір: 1080p (обов’язково), 1440p, 2160p.
	•	Кодек: libvpx-vp9 -b:v 0 -crf 30 -pix_fmt yuv420p -row-mt 1 -tile-columns 2 -threads 0 -ан.
DoD: згенеровані wallpaper_1080.webm (і інші, якщо обрані) з правильним FPS/аспектом.

Етап 6 — Poster & Packaging

Завдання
	•	Постер: кадр на 2-й секунді або в середині сегмента (що менше) → poster.jpg.
	•	metadata.json: { "name", "sizes": [1080,…], "duration": <s> }.
	•	ZIP структура: wallpaper_1080.webm, wallpaper_1440.webm?, wallpaper_2160.webm?, poster.jpg, metadata.json.
	•	Після успіху: reveal у Finder → open -a "Opera GX" (якщо встановлено).
DoD: ZIP створюється, відкривається в Finder; Opera GX активується.

Етап 7 — UX, хоткеї, блокування виходу

Завдання
	•	Кнопки: Install (primary), Close (secondary).
	•	Хоткеї: Enter=Install; E/Esc/⌘Q=Close, але під час експорту — заблоковано.
	•	Прогрес: рядок статусу/лейбл без %: Exporting 1080p…, Packaging….
DoD: хоткеї працюють; під час експорту — неможливо закрити.

Етап 8 — Продуктивність і тимчасові файли

Завдання
	•	Обмеження: вхід ≤ 2 ГБ; RAM пікова ≤ 400 МБ на розмір.
	•	Експорт по одному розміру за раз; очищення temp як при успіху, так і при помилці.
DoD: після будь-якого run temp-папка відсутня.

Етап 9 — Повідомлення/помилки

Завдання
	•	Лаконічні тости про помилки; після фейлу — кнопки знову активні.
	•	Текстові статуси по кроках експорту/пакування.
DoD: інциденти не блокують UI назавжди; відновлення стану працює.

Етап 10 — Security/Privacy перевірки

Завдання
	•	Нуль мережевих викликів від вебв’ю (ні fetch/XHR/WebSocket).
	•	CSP/allowlist у Tauri: лише потрібні capabilities.
DoD: моніторинг проксі/mitm → зовнішніх запитів немає.

Етап 11 — Build & Release

Завдання
	•	pnpm tauri build → підписаний/нотаризований .dmg (Apple Dev ID).
	•	Додати ліцензії для ffmpeg (LICENSES/ffmpeg.txt).
	•	README: setup/dev/build + скрипт швидкої перевірки ffprobe.
DoD: інсталятор запускається на чистій macOS 12+ (arm64), sidecars працюють.

Етап 12 — Acceptance (smoke)
	1.	Імпорт MP4/MOV/WEBM/GIF → метадані показані, без раннього “unsupported”.
	2.	70 s → Trim видно; 12 s → Trim приховано.
	3.	Експорт 1080p → ZIP імпортується в Opera GX; відео лупиться, без аудіо.
	4.	Під час експорту вихід заблокований; після — Finder+Opera активуються.
	5.	Вікно 350×650, не ресайзиться; хоткеї працюють.
	6.	У dev/prod — відсутні мережеві запити від додатку.

⸻

4) Файлова структура (мінімум)

repo/
  docs/PLAN.md
  LICENSES/ffmpeg.txt
  index.html
  src/
    main.ts
    ui.ts
    lib/
      types.ts
      ff.ts
      state.ts
  src-tauri/
    tauri.conf.json
    bin/
      ffmpeg
      ffprobe
    src/
      main.rs
      commands.rs


⸻

5) Інтерфейси та константи

Types

export type VideoMeta = { path: string; duration: number; width: number; height: number; fps: number; vfr: boolean; codec: string };
export type ExportRequest = { path: string; start: number; end: number; sizes: number[]; name: string };

Константи
	•	Розміри: [1080, 1440, 2160] (checkboxes).
	•	Якість VP9: CRF = 30.
	•	Обмеження Trim: 5…60 s.
	•	Target FPS: min(30, round(input_fps)).

⸻

6) Шаблони команд (референс)

ffprobe

ffprobe -v error -select_streams v:0 \
  -show_entries stream=codec_name,width,height,avg_frame_rate,r_frame_rate,nb_frames \
  -show_entries format=duration -of json "<path>"

ffmpeg (на один розмір H)

# input_fps > 30 → 30 CFR
ffmpeg -y -ss <start> -to <end> -i "<in>" \
  -vf "scale=-2:<H>,crop=iw:trunc(iw*9/16),fps=30" \
  -c:v libvpx-vp9 -b:v 0 -crf 30 -pix_fmt yuv420p -row-mt 1 -tile-columns 2 -threads 0 -an \
  -r 30 -vsync cfr "<tmp>/wallpaper_<H>.webm"

# input_fps ≤ 30 → без апсемплу
ffmpeg -y -ss <start> -to <end> -i "<in>" \
  -vf "scale=-2:<H>,crop=iw:trunc(iw*9/16)" \
  -c:v libvpx-vp9 -b:v 0 -crf 30 -pix_fmt yuv420p -row-mt 1 -tile-columns 2 -threads 0 -an \
  -r <TARGET_FPS> -vsync cfr "<tmp>/wallpaper_<H>.webm"

Poster

ffmpeg -y -ss <poster_ts> -i "<in>" -frames:v 1 -q:v 2 "<tmp>/poster.jpg"

Activate Opera GX

open -a "Opera GX" || true


⸻

7) Ризики та пом’якшення
	•	VP9 performance: на довгих кліпах час експорту ↑ → обмежуємо сегмент до 60 s; послідовна обробка розмірів.
	•	Відмінності FPS/VFR: жорсткий CFR у виході, явний маркер VFR у UI.
	•	Нотаризація macOS: переконатися, що sidecars підписані/в пакеті; за потреби — ad-hoc підпис.
	•	Вирізання 16:9: можливі втрати важливого контенту по краях → повідомлення у UI про автокадрування.

⸻

8) Out of scope (MVP)
	•	Вставки аудіо, гучність, LUTи.
	•	Розширені налаштування якості/CRF/битрейту.
	•	Багатомовність UI.

⸻

9) Checklists

Перед merge етапу
	•	Tests/Acceptance пройдені локально.
	•	Temp очищається в success/fail.
	•	Жодних мережевих викликів.
	•	README оновлено.

Перед релізом
	•	.dmg підписано і нотаризовано.
	•	ffmpeg ліцензії додано.
	•	Smoke-тести на чистій macOS 12+ (arm64).

