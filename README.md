# tracker

Time Tracker — десктопное приложение (Tauri 2, Rust + Vue 3/Vuetify). Описание функционала: `ОПИСАНИЕ_ФУНКЦИОНАЛА.txt`.

## Разработка

```bash
npm install
npm run dev:tauri:win   # запуск приложения (Windows)
npm run lint            # ESLint
npm run format:check    # Prettier (npm run format — исправить)
npm run build           # сборка фронтенда
```

Проверки Rust (из `src-tauri/core` и `src-tauri`):

```bash
cargo clippy --all-targets -- -D warnings
cargo test              # только в src-tauri/core
```

Те же проверки выполняет CI (`.github/workflows/ci.yml`).

## Структура

- `src-tauri/core` — логика без Tauri: хранилище, справочники, отчёты, бэкапы, тесты.
- `src-tauri/src` — команды Tauri (`commands.rs`), трей и напоминания (`tray.rs`).
- `src/stores` — Pinia: `app` (загрузка, ошибки), `settings`, `dictionaries`, `columns`, `tasks`.
- `src/composables` — ресайз колонок и выбор строк таблицы.
