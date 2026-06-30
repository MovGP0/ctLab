---
name: atopile
description: Work with Atopile and the ato toolchain for electronics design as code. Use when creating or editing .ato files, ato.yaml manifests, modules, interfaces, packages, component picking, KiCad layout integration, manufacturing exports, package publishing, or debugging ato build/config/dependency workflows.
---
# Atopile

Use this skill for Atopile projects: `.ato` source files, `ato.yaml`, package dependencies, part creation, KiCad layout sync, manufacturing exports, and package publishing.

Atopile is a language, compiler, and toolchain for circuit-board design. The `ato` DSL describes electronic modules, interfaces, components, units, tolerances, connections, and assertions. The `ato` CLI compiles the design, solves constraints, picks components, runs checks, updates KiCad layout files, and generates outputs.

Start with the relevant reference:

| Task area | Read |
| --- | --- |
| Install the extension, CLI, KiCad, or development checkout | [Setup](references/setup.md) |
| Write or review `.ato` language constructs | [Ato Language](references/ato-language.md) |
| Run builds, understand compiler flow, or debug CLI usage | [Compiler And CLI](references/compiler-cli.md) |
| Edit `ato.yaml`, project paths, builds, or checks | [Project Structure And Config](references/project-structure-config.md) |
| Add passives, exact components, custom parts, or packages | [Components And Packages](references/components-packages.md) |
| Work with KiCad layout sync or manufacturing exports | [Layout And Manufacturing](references/layout-manufacturing.md) |
| Publish reusable Atopile packages | [Publishing Packages](references/publishing.md) |
| Look up built-in component/interface names | [API Reference Index](references/api-reference-index.md) |

## Core Rules

- Treat `.ato` code as the source of circuit intent. Preserve units, tolerances, assertions, and interface types when making changes.
- Keep generated or tool-managed files separate from source edits. Inspect project conventions before changing generated KiCad layout, `.ato/modules`, `.ato/`, or `build/` artifacts.
- Prefer official package modules from `packages.atopile.io` or existing project dependencies before creating a custom component.
- When a physical part is required, decide whether the design can use auto-picked passives, `ato create part`, a registry package, or a manually defined custom part.
- Validate with `ato build` when possible. Use `ato --help` and command-specific `--help` for the installed version.
- For layout work, remember that Atopile updates the KiCad project from code, while placement/routing still happens in KiCad. Use the Atopile KiCad plugin to sync reusable package layouts.

## Common Commands

```powershell
ato --version
ato --help
ato build
ato add atopile/rp2040
ato sync
ato remove atopile/rp2040
ato create part
ato configure
```

Use `ato -v build` when verbose compiler output is needed. Put `-v` right after `ato`, before the subcommand.

## Validation Checklist

- `ato.yaml` has the intended `paths.src`, `paths.layout`, and `builds.<name>.entry`.
- Imports use the current syntax: `from "where.ato" import What`.
- Values that represent physical quantities include units and appropriate tolerances.
- Interfaces of the same type are connected with `~`.
- Constraints use `assert` with `is`, `<`, `>`, or `within` as appropriate.
- Dependencies listed in `ato.yaml` are installed with `ato sync`.
- KiCad is installed and `ato configure` has been run when layout/plugin behavior is involved.
- Manufacturing exports are reviewed for Gerbers, BOM, pick-and-place, and part availability.

## Official Sources

- Atopile docs index: https://docs.atopile.io/llms.txt
- Atopile docs home: https://atopile.io/
- Atopile GitHub repository: https://github.com/atopile/atopile
- Package registry: https://packages.atopile.io/
