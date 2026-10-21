# Time Bridge

A menu bar clock for Budapest ↔ Bali (plus Helsinki, San Francisco, Jakarta and Papua).

- **Left-click** the icon in the menu bar → the clock drops down under it.
- **Click anywhere else** → it tucks away.
- **Right-click** the icon → *Open at Login* and *Quit*.

## Install
1. Open the latest release on this repo's **Releases** page and download the `.dmg`.
2. Open it and drag **Time Bridge** into **Applications**.
3. First launch: right-click Time Bridge → **Open** → **Open**.
   (On newer macOS: if it's blocked, go to System Settings → Privacy & Security → **Open Anyway**.)

## Changing the clock
The whole clock is `dist/index.html`. Edit it, bump `version` in `src-tauri/tauri.conf.json`,
push to `main`, and GitHub builds a new `.dmg` automatically.
