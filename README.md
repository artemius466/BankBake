# BankBake

BankBake is a lightweight desktop tool that compiles audio files directly into FMOD `.bank` files without needing to set up or manage an FMOD Studio project.

## Usage

1. Set a **Bank Name** and **Output Folder**.
2. Drag and drop audio files (`.wav`, `.ogg`, `.mp3`, `.flac`, `.aiff`, `.wma`) into the window.
3. Edit the event path (e.g. `event:/sfx/player/jump` or `event:/music/level1`), pick **SFX** or **Music**, and toggle **Loop** if needed.
4. *(Optional)* Open **Advanced** on any track to adjust volume (dB) or set custom loop start/end points.
5. Click **Bake Bank** to generate `<BankName>.bank`, `Master.bank`, `Master.strings.bank`, and `GUIDs.txt`.

## Building from Source

Requires [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), and `pnpm`.

```powershell
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build portable release (outputs to release/BankBake-Portable-win-x64.zip)
pnpm portable
```

## Tech Stack

- **Backend**: Rust, Tauri 2
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, WaveSurfer.js
- **Compiler**: FMOD Studio 2.02 CLI (`fmodstudiocl.exe`)

## License

[MIT](LICENSE)
