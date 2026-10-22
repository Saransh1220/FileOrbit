## Beta

An early build for **Windows** (64-bit, tried on Windows 11) and **macOS** (Apple Silicon and Intel in one app). **Neither is signed**, so your system will warn you the first time. Everything runs on your computer; the only downloads are the converter programs you choose.

### Windows

1. Download **Radial-Convert-windows-x86_64-setup.exe** below and run it. It installs for your user only (no administrator rights needed).
2. Windows SmartScreen will warn: choose *More info*, then *Run anyway*.
3. Start *Radial Convert* from the Start menu. It lives in the system tray; there is no main window on purpose.
4. Hold **Shift**, drag a file out of Explorer (or from the desktop) and drop it on a format in the ring.

### macOS

1. Download **Radial-Convert-macos-universal.dmg** (or the `.zip`), open it and drag *Radial Convert* to Applications.
2. The app is not signed or notarized, so macOS will refuse to open it at first. Open it once with **right-click, Open**, or go to *System Settings, Privacy & Security* and press *Open Anyway*. If it says the app is "damaged", run `xattr -dr com.apple.quarantine "/Applications/Radial Convert.app"` in Terminal.
3. Grant **Input Monitoring** when asked (*System Settings, Privacy & Security, Input Monitoring*). Without it the app cannot notice Shift+drag. Quit and start the app again after allowing it.
4. Hold **Shift**, drag a file out of Finder and drop it on a format in the ring. The app lives in the menu bar.

### Both

- The first time a file needs a converter, the app offers to install it (PDF tools 4 MB, Audio 77 MB, Documents 375 MB…). You can also manage them from the tray / menu-bar icon, *Manage converters*.
- Optional: the `radial` command-line installer for converters (`radial install`, `radial list`, `radial doctor`): `radial-windows-x86_64.exe`, `radial-macos-aarch64` or `radial-macos-x86_64`.
- Check any download against `SHA256SUMS` if you like (Windows: `Get-FileHash <file>`; macOS: `shasum -a 256 <file>`).

### What to try

- Shift+drag a `.docx`, `.mp3`, `.png` or `.pdf` and convert it.
- Select several PDFs and Shift+drag for **Merge** and **Organize** (the page editor).
- Shift+drag pictures and choose **Resize** to shrink or compress them.
- Try the themes from the tray / menu-bar *Theme* menu.

### Known limits

- Windows: tried by hand on one PC. **Not yet tried:** several monitors with different scaling, network folders, the Text pack (pandoc).
- macOS: the release build is made by GitHub's build service and **has not been opened on a Mac yet**; the app was run on a Mac during development, not this exact file.
- Windows: Shift+drag does not work from an Explorer window that was started as administrator (a Windows rule).
- Downloads are the slow part (the Documents pack is 375 MB); an interrupted download continues where it stopped.
- On Windows the converter data lives in the same folder as the app, so uninstalling removes the downloaded converters too.
- PDF to Word keeps the text but the layout can differ.

Please report problems at https://github.com/Saransh1220/FileOrbit/issues with your system, what you dragged, what you chose, and any message the toast showed.
