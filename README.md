# TidaLuna Launcher

> [!NOTE]
> You probably should just use the [official CLI installer](https://github.com/jxnxsdev/TidaLuna-Installer) for this as it can achieve the same thing. I wasn't aware of that when I made this so this is more just a learning project for me now.

A launcher for [TidaLuna](https://github.com/Inrixia/TidaLuna), a client mod for the TIDAL music app.

The launcher automatically downloads and installs TidaLuna into your TIDAL installation, then launches TIDAL with the mod active.

## Windows

Download and run [the latest installer](https://github.com/Johnnycyan/Tidaluna-Launcher/releases/latest/download/TidaLunaInstaller.exe).

## Linux

```
sh -c "$(curl -fsSL https://github.com/Johnnycyan/Tidaluna-Launcher/releases/latest/download/install.sh)"
```

### Uninstalling

```
sh -c "$(curl -fsSL https://github.com/Johnnycyan/Tidaluna-Launcher/releases/latest/download/install.sh)" -- --uninstall
```

## macOS

Working on it...

## How It Works

1. Finds your TIDAL installation
2. Downloads the latest [TidaLuna release](https://github.com/Inrixia/TidaLuna/releases/latest) (`luna.zip`)
3. Backs up `app.asar` → `original.asar` in TIDAL's resources directory
4. Extracts `luna.zip` into a `resources/app/` folder
5. Launches TIDAL with TidaLuna active

## Notes

- The Windows Store version of TIDAL is **not supported**. Please install the [desktop version](https://offer.tidal.com/download).
- Make sure TIDAL is closed before running the launcher for installation.
