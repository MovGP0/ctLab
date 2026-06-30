# Project Structure And Config

Sources:

- https://docs.atopile.io/atopile-0.14.x/essentials/5-project-structure.md
- https://docs.atopile.io/atopile-0.14.x/reference/config.md

## Recommended Project Layout

```text
<project_name>/
  .ato/                 # Cached dependencies
  build/                # Build outputs
  ato.yaml              # Project manifest
  elec/
    layouts/            # Board layout files
    src/                # Ato source and parts
      <project_name>.ato
      parts/            # Part info, pinout, footprint, 3D model
```

The docs describe the manifest as the central file read by the compiler and package manager.

Keep one module per file where possible. Keep source files, layout files, dependencies, and generated outputs distinct.

## Minimal Manifest Pattern

```yaml
requires-atopile: "^0.10.8"

paths:
  src: ./src
  layout: ./layout

builds:
  default:
    entry: main.ato:App
    hide_designators: true
    exclude_checks: ["PCB.requires_drc_check"]

dependencies:
  - type: registry
    identifier: atopile/ti-ads1115
    release: 0.1.6
```

Some reference pages call the compiler-version field `ato-version`, while project examples use `requires-atopile`. Preserve the convention already used by the project unless intentionally migrating config format.

## Config Fields

| Field | Purpose |
| --- | --- |
| `ato-version` / `requires-atopile` | Compiler version compatibility, depending on project format. |
| `paths.src` | Source code directory. Default documented as `elec/src`. |
| `paths.layout` | Layout directory where KiCad projects are stored and searched. Default documented as `elec/layout`. |
| `dependencies` | Project dependencies managed by `ato add`, `ato sync`, and `ato remove`. |
| `builds` | Named build targets. |
| `builds.<name>.entry` | Required build entry point, such as `main.ato:App`. |
| `builds.<name>.targets` | Build output targets. Defaults to `["__default__"]`. |
| `builds.<name>.exclude_targets` | Targets to skip, such as `bom` or `mfg-data`. |
| `builds.<name>.exclude_checks` | Checks to skip by qualified name. |
| `builds.<name>.hide_designators` | Hide all designators from the built PCB. |

Examples of excluded checks:

```yaml
builds:
  default:
    exclude_checks:
      - PCB.requires_drc_check
      - I2C.requires_unique_addresses
      - requires_external_usage
```

## Typical Workflow

1. Sketch the circuit.
2. Search `packages.atopile.io` and GitHub for existing modules.
3. Install packages with `ato add`.
4. Design modules and calculations in `.ato`.
5. Run `ato build` to compile, pick components, and update the layout.
6. Use KiCad to place and route.
7. Repeat code/build/layout until the design is ready.
8. Generate or collect manufacturing files.
