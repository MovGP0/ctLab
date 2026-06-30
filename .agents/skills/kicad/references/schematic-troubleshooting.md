# Schematic Troubleshooting

## Save-Crash And Parse Errors

If KiCad reports:

```text
Expecting "(" in line 1 offset 1
```

first check whether the file was truncated to 0 bytes. A save crash can leave an empty `.kicad_sch`, which produces the same first-token parse error as an encoding problem.

For a valid `.kicad_sch`, the first bytes should start directly with:

```text
28 6B 69
```

That is `(ki...`. A UTF-8 BOM would appear before the opening parenthesis and should not be present in schematic files.

## Generated Schematic Writer Compatibility

A schematic can be readable or exportable but still crash KiCad's writer if generated symbol instances contain malformed `instances` blocks.

Known bad pattern:

```lisp
(project "project" (path "/" ...))
```

KiCad CLI may still export SVG, but `sch upgrade --force` and the GUI save path can truncate the schematic.

Prefer letting KiCad regenerate valid placed-symbol `instances` blocks instead of hand-generating placeholder ones. Valid regenerated blocks use the actual project name and sheet UUID path:

```lisp
(project "ctLab" (path "/<root-sheet-uuid>" ...))
```

## Isolation Test

Use a copy, not the original:

```powershell
New-Item -ItemType Directory -Force .\.temp | Out-Null
Copy-Item .\KiCad\example.kicad_sch .\.temp\example.kicad_sch
kicad-cli sch upgrade --force .\.temp\example.kicad_sch
Get-Item .\.temp\example.kicad_sch | Select-Object Length
```

If the command fails and the copy becomes 0 bytes, the problem is accepted by the reader but rejected by the writer.

## Recovery Pattern

When recovering from truncation:

1. Restore the schematic from version control before opening it again in KiCad.
2. Remove malformed generated `instances` data.
3. Keep `embedded_fonts` as a schematic-level setting. Do not include schematic-level fields inside individual `lib_symbols` definitions.
4. Verify with `kicad-cli sch export svg`.
5. Verify writer compatibility with `kicad-cli sch upgrade --force` on a copy.
