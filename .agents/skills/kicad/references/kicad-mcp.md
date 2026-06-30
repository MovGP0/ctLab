# KiCad MCP

The KiCad MCP server lets MCP-compatible AI clients inspect or interact with KiCad through KiCad's IPC API.

## Prerequisites

- KiCad 9 or newer.
- Python 3.11 or newer.
- `uv`, for running Python tools in isolated environments.
- KiCad must be running with IPC enabled before the MCP server can talk to an editor.

Install `uv` on Windows:

```powershell
winget install --id astral-sh.uv --exact
```

## Run Without Installing

Run the schematic MCP server:

```powershell
uvx kicad-mcp --editor-type schematic
```

Run the PCB MCP server:

```powershell
uvx kicad-mcp --editor-type pcb
```

Other supported editor types are `symbol` and `footprint`.

## MCP Client Configuration

For clients using the common `mcpServers` format:

```json
{
  "mcpServers": {
    "Kicad-Schematic-MCP": {
      "type": "stdio",
      "command": "uvx",
      "args": [
        "kicad-mcp",
        "--editor-type",
        "schematic"
      ]
    },
    "Kicad-PCB-MCP": {
      "type": "stdio",
      "command": "uvx",
      "args": [
        "kicad-mcp",
        "--editor-type",
        "pcb"
      ]
    }
  }
}
```

For VS Code-style MCP configuration, use `servers` instead of `mcpServers`:

```json
{
  "servers": {
    "Kicad-Schematic-MCP": {
      "type": "stdio",
      "command": "uvx",
      "args": [
        "kicad-mcp",
        "--editor-type",
        "schematic"
      ]
    }
  }
}
```

Restart the MCP client after changing the configuration.

## Development Install

Use this when editing or debugging the MCP server itself:

```powershell
git clone https://github.com/lamaalrajih/kicad-mcp.git
Set-Location .\kicad-mcp
uv sync
uv run kicad-mcp --editor-type schematic
```

For a local development MCP client entry:

```json
{
  "servers": {
    "kicad-schematic-mcp-dev": {
      "type": "stdio",
      "command": "uv",
      "args": [
        "--directory",
        "D:/Tools/kicad-mcp",
        "run",
        "kicad-mcp",
        "--editor-type",
        "schematic"
      ]
    }
  }
}
```

Replace `D:/Tools/kicad-mcp` with the actual checkout path.

## Sources

- https://pypi.org/project/kicad-mcp/
- https://github.com/lamaalrajih/kicad-mcp
