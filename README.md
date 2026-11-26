# BankBake

BankBake is a lightweight desktop tool that compiles audio files directly into FMOD `.bank` files without needing to set up or manage an FMOD Studio project.

## Usage

1. Set a **Bank Name** and **Output Folder**.
2. Drag and drop audio files (`.wav`, `.ogg`, `.mp3`, `.flac`, `.aiff`, `.wma`) or an existing `.bankbake` project file into the window.
3. Edit the event path (e.g. `event:/sfx/player/jump` or `event:/music/level1`), pick **SFX** or **Music**, and toggle **Loop** if needed.
4. *(Optional)* Open **Advanced** on any track to adjust volume (dB) or set custom loop start/end points.
5. Click **Bake Bank** to generate `<BankName>.bank`, `Master.bank`, `Master.strings.bank`, `GUIDs.txt`, and an editable `<BankName>.bankbake` project archive containing all audio assets and settings.

## Building from Source

Requires [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/), and `pnpm`.

```sh
# Install dependencies
pnpm install --frozen-lockfile

# Run in development mode
pnpm tauri dev
```

### Linux

Install the Tauri system dependencies for your distribution. On Arch-based systems the desktop build uses `base-devel` and `webkit2gtk-4.1`. [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/#system-dependencies)

BankBake supports two compiler setups on Linux.

- Install FMOD Studio 2.02 for Linux. BankBake looks for `fmodstudiocl` on `PATH` and in `/opt/fmodstudio`. For another location, set **FMOD Studio CLI Path** in Settings to the executable or its containing folder. [FMOD downloads](https://www.fmod.com/download/design.html)
- On x86-64, install Wine with `wine` and `winepath` on `PATH`. If BankBake cannot find a native compiler, it uses the bundled `fmodstudio/fmodstudiocl.exe` through Wine. Wine may initialize its prefix on the first bake.

To try a bake, load the bundled samples, choose a writable output folder, and click **Bake Bank**. The Rust end-to-end bake test uses automatic compiler discovery. For a native compiler in a custom location, add its directory to `PATH` before running the test.

```sh
cargo test --manifest-path src-tauri/Cargo.toml test_discover_compiler_and_bake_bank -- --ignored
```

### Portable builds

Run `pnpm portable` to package the app in `release/`. Windows produces `BankBake-Portable-v<version>-win-x64.zip`. Linux uses `tar` to produce `BankBake-Portable-v<version>-linux-<arch>.tar.gz`. Extract the Linux archive and run `./bankbake` from the extracted folder. The target system needs GTK 3, WebKitGTK 4.1, and one of the compiler setups above.

## Tech Stack

- **Backend**: Rust, Tauri 2
- **Frontend**: Svelte 5, TypeScript, Tailwind CSS, WaveSurfer.js
- **Compiler**: FMOD Studio 2.02 CLI

## License

[MIT](LICENSE)
