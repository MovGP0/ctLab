---
name: kicad
description: Work with KiCad projects, schematics, symbol libraries, PCB files, KiCad MCP, NGSpice simulation, and PCB trace-width calculations. Use when editing or reviewing .kicad_sch, .kicad_pcb, .kicad_sym, .kicad_pro, KiCad library tables, schematic symbols, power nets, grid alignment, generated schematics, simulation setup, or KiCad installation/tooling issues.
---
# KiCad

Use this skill for KiCad project work, especially when the task touches schematic files, generated KiCad data, symbol/library setup, KiCad MCP, simulation, or PCB sizing calculations.

Start by identifying which KiCad surface is involved:

| Task area | Read |
| --- | --- |
| Installing KiCad or configuring database libraries | [Setup](references/setup.md) |
| Connecting an agent through KiCad MCP | [KiCad MCP](references/kicad-mcp.md) |
| Reading or editing `.kicad_sch` files | [Schematic Files](references/schematic-files.md) |
| Looking up complete schematic file-format element documentation | [Copied Schematics Docs](references/Schematics/KiCAD Schematic Files.md) |
| Placing symbols, power symbols, wires, labels, sheets, or properties | [Schematic Editing](references/schematic-editing.md) |
| Diagnosing KiCad save crashes, parse errors, or generated schematic problems | [Schematic Troubleshooting](references/schematic-troubleshooting.md) |
| Running KiCad simulation or installing NGSpice | [Simulation](references/simulation.md) |
| Calculating PCB trace width, voltage drop, resistance, or power loss | [PCB Trace Width](references/pcb-trace-width.md) |

## Core Rules

- Treat KiCad files as structured S-expression data. Preserve syntax, balanced parentheses, UUIDs, sheet paths, and KiCad-generated fields unless the task explicitly requires changing them.
- Prefer changing the smallest relevant KiCad file set. Avoid touching project metadata, history files, or editor state unless the user asks for that layer.
- For `.kicad_sch` files, preserve UTF-8 without BOM and verify the first token still starts with `(kicad_sch`.
- Keep schematic placement on the design grid. In the current local KiCad notes, the important schematic grid is often `1.27 mm` / `50 mil`.
- When editing placed symbols, distinguish the symbol definition in `(lib_symbols ...)` from placed schematic `(symbol ...)` instances.
- When changing visible component references, update both placed `property "Reference"` values and matching instance `(reference "...")` values. Do not rewrite `lib_id` names just because the visible reference changed.
- Use KiCad itself or `kicad-cli` for validation when available. For generated schematics, validate both read/export behavior and writer behavior.

## Full Schematic Reference Corpus

The source KiCad schematic documentation from `D:\GitHub\Obsidian_Electronics\KiCAD\Schematics` is copied into `references/Schematics/`. Use that folder when the compact references here are not enough, especially for exact S-expression element details under `references/Schematics/Elements/` or standard-library symbol notes under `references/Schematics/Symbols/`.

## Common Validation

Use the fastest validation that matches the task:

```powershell
kicad-cli sch export svg .\KiCad\example.kicad_sch --output .\.temp\kicad-export
```

For generated schematic writer compatibility, run `sch upgrade --force` only on a copy:

```powershell
Copy-Item .\KiCad\example.kicad_sch .\.temp\example.kicad_sch
kicad-cli sch upgrade --force .\.temp\example.kicad_sch
Get-Item .\.temp\example.kicad_sch | Select-Object Length
```

Clean up `.temp/` before finishing unless the user asks to keep artifacts.
