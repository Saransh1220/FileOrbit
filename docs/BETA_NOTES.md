## Windows beta

An early build for 64-bit Windows. It has only been tried on Windows 11. **It is not signed**, so Windows SmartScreen will warn you the first time: choose *More info*, then *Run anyway*. Everything runs on your computer; the only downloads are the converter programs you choose.

### Install

1. Download **Radial-Convert-windows-x86_64-setup.exe** below and run it. It installs for your user only (no administrator rights needed).
2. Start *Radial Convert* from the Start menu. It lives in the system tray; there is no main window on purpose.
3. Hold **Shift**, drag a file out of Explorer (or from the desktop) and drop it on a format in the ring.
4. The first time a file needs a converter, the app offers to install it (PDF tools 4 MB, Audio 77 MB, Documents 375 MB…). You can also manage them from the tray icon, *Manage converters*.

Optional: `radial-windows-x86_64.exe` is the command-line installer for converters (`radial install`, `radial list`, `radial doctor`).

Check the file against `SHA256SUMS` if you like: `Get-FileHash .\Radial-Convert-windows-x86_64-setup.exe`.

### What to try

- Shift+drag a `.docx`, `.mp3`, `.png` or `.pdf` and convert it.
- Select several PDFs and Shift+drag for **Merge** and **Organize** (the page editor).
- Shift+drag pictures and choose **Resize** to shrink or compress them.
- Try the themes from the tray's *Theme* menu.

### Known limits

- Tried by hand on one Windows 11 PC. **Not yet tried:** several monitors with different scaling, network folders, the Text pack (pandoc).
- Shift+drag does not work from an Explorer window that was started as administrator (a Windows rule).
- Downloads are the slow part (the Documents pack is 375 MB); an interrupted download continues where it stopped.
- The converter data lives in the same folder as the app, so uninstalling removes the downloaded converters too.
- PDF to Word keeps the text but the layout can differ.
- macOS is not part of this beta.

Please report problems at https://github.com/Saransh1220/FileOrbit/issues with what you dragged, what you chose, and any message the toast showed.
