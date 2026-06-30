# Setup

Sources:

- https://docs.atopile.io/atopile-0.14.x/quickstart/1-installation.md
- https://github.com/atopile/atopile

## Preferred Local Workflow

The primary local workflow is the Atopile extension for VS Code-compatible editors:

- VS Code: https://code.visualstudio.com/download
- Cursor: https://cursor.com/download
- Google Antigravity: https://antigravity.google/download
- Extension: https://marketplace.visualstudio.com/items?itemName=atopile.atopile

After installing the extension, let it finish installing `uv` if prompted. The extension adds an Atopile sidebar and one-click project/build controls.

For a browser-only trial, use:

- https://playground.atopile.io/

The playground does not persist work between sessions.

## KiCad

KiCad is required for layout/routing work. Builds can update `.kicad_pcb` without opening KiCad, but you need KiCad for board layout.

```powershell
winget install --id 'KiCad.KiCad' --exact
```

Other documented install commands:

```bash
brew install kicad
sudo pacman -S kicad
sudo apt install kicad
```

## CLI Installation

On macOS, Homebrew is recommended:

```bash
brew install atopile/tap/atopile
```

On other platforms, install with `uv`:

```powershell
uv tool install atopile
ato --version
```

If this is the first `uv tool install`, `uv` may print another command to finish shell setup. Run it before expecting `ato` to be on `PATH`.

## Development Checkout

Use this when working on Atopile itself:

```powershell
git clone https://github.com/atopile/atopile
Set-Location .\atopile
uv sync --dev
```

The upstream repo documents development tests with:

```bash
pytest -q
```

Atopile compatibility notes from the upstream README:

- OS: macOS, Linux, Windows with WSL recommended.
- Editors: VS Code or Cursor.
- EDA: KiCad recommended for layout, not required to start.
