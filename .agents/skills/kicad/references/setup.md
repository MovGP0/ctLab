# Setup

## Windows

Install KiCad with WinGet:

```powershell
winget install --id 'KiCad.KiCad' --exact
```

## SQLite ODBC Driver For Database Libraries

KiCad database libraries (`*.kicad_dbl`) need a matching ODBC driver on Windows. The local CERN library notes use:

```text
D:\Obsidian_Electronics\cern-kicad-libs\CERN_Windows.kicad_dbl
```

That file expects this 64-bit ODBC driver name:

```text
SQLite3 ODBC Driver
```

Install the 64-bit Windows installer from Christian Werner's SQLite ODBC project:

- https://www.ch-werner.de/sqliteodbc/
- `sqliteodbc_w64.exe`

Run the installer as administrator. A silent install from an elevated shell is:

```powershell
.\sqliteodbc_w64.exe /S
```

Verify registration:

```powershell
Get-ItemProperty 'HKLM:\SOFTWARE\ODBC\ODBCINST.INI\ODBC Drivers' |
  Select-Object -ExpandProperty 'SQLite3 ODBC Driver'
```

Expected output:

```text
Installed
```

After installing the driver, restart KiCad and add `CERN_Windows.kicad_dbl` as a symbol library with format `Database`.

On Windows ARM64, check the KiCad and ODBC driver architectures. The `sqliteodbc_w64.exe` installer provides an x64 driver, not a native Windows ARM64 driver. Native ARM64 KiCad cannot load that x64 ODBC DLL and may fail with Windows system error 193.

Workarounds:

- Install and run x64 KiCad so it can load the x64 SQLite ODBC driver.
- Skip the database library and add `D:\Obsidian_Electronics\cern-kicad-libs\SchLib\*.kicad_sym` as normal `KiCad` symbol libraries.

## Ubuntu

Install the current stable KiCad release from the official KiCad PPA:

```bash
sudo add-apt-repository --yes ppa:kicad/kicad-10.0-releases
sudo apt update
sudo apt install --install-recommends kicad
```

For KiCad 9.x specifically:

```bash
sudo add-apt-repository --yes ppa:kicad/kicad-9.0-releases
sudo apt update
sudo apt install --install-recommends kicad
```

## macOS

Install with Homebrew Cask:

```bash
brew install --cask kicad
```

Alternatively, download the official installer from:

- https://www.kicad.org/download/macos/
