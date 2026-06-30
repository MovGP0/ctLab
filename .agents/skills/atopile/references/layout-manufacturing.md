# Layout And Manufacturing

Sources:

- https://docs.atopile.io/atopile-0.14.x/essentials/6-layout.md
- https://docs.atopile.io/atopile-0.14.x/quickstart/4-manufacturing-files.md

## Layout Model

`.ato` code defines circuit function through modules, interfaces, constraints, and connections. Physical components still need placement and routing.

Atopile uses KiCad for layout. `ato build` generates or updates the KiCad project files. If there is only one build target, `ato` may open the KiCad file automatically.

## KiCad Plugin

The `ato` compiler automatically installs a KiCad plugin to help with layout. If it is missing, run:

```powershell
ato configure
```

## Reusing Package Layouts

Packages can include reusable layouts. Example:

```ato
from "rp2040/RP2040Kit.ato" import RP2040Kit

module App:
    uc = new RP2040Kit
```

To reuse a layout from a package:

1. Run `ato build` so layout syncs with code.
2. Open KiCad.
3. Use the Atopile KiCad plugin.
4. Click `Sync Group`.
5. Select the group.
6. Use `Pull` to bring in the layout from the module's KiCad layout file.

The compiler maps layouts with a class or super-class that has a build. To create a reusable layout for your own class, add a build config with `ato create build` and point its entry at the module.

## Manual Layout Work

Use KiCad to place and route the rest of the board. Keep source-of-truth boundaries clear:

- `.ato` defines function, interfaces, constraints, and component intent.
- KiCad layout stores physical placement, routing, board outline, and layout-specific details.
- `ato build` updates the PCB from the code model.

## Manufacturing Export

The extension has a `Generate manufacturing data` workflow. It has three stages:

1. Build: checks for uncommitted changes and builds the project.
2. Review: inspect Gerbers, BOM, and 3D preview.
3. Export: choose an output directory and export production files.

Generated export files include:

- Gerber ZIP for fabrication.
- BOM CSV for procurement.
- Pick-and-place CSV for assembly.

The documented export flow also runs post-build checks that confirm Gerbers, BOM, and pick-and-place files were generated and checks part availability. Outputs are intended to be ready for JLCPCB upload.

## Agent Validation Notes

- If changing code that should affect layout, run `ato build`.
- If layout sync does not appear in KiCad, run `ato configure`.
- For manufacturing tasks, verify Gerber ZIP, BOM CSV, and pick-and-place CSV exist.
- Do not treat generated manufacturing outputs as source unless the user explicitly asks to update committed release artifacts.
