# Components And Packages

Sources:

- https://docs.atopile.io/atopile-0.14.x/essentials/3-add-component.md
- https://docs.atopile.io/atopile-0.14.x/essentials/4-packages.md
- https://packages.atopile.io/

## Ways To Add Components

Atopile documents four approaches:

1. Auto-pick passive components.
2. Add a specific component with `ato create part`.
3. Use a package from the registry.
4. Create a custom part manually.

## Auto-Picked Passives

Use this for passives such as resistors, capacitors, and inductors.

```ato
resistor = new Resistor
resistor.resistance = 10kohm +/- 5%
resistor.package = "0402"
```

On build, Atopile can automatically select a part that satisfies the constraints.

## Specific Component With `ato create part`

Use:

```powershell
ato create part
```

The prompt accepts:

- A JLCPCB part number, for example `C7426`.
- An exact manufacturer part number, for example `NE5532DR`.

Generated component files include traits such as:

- `is_auto_generated`
- `is_atomic_part`
- `has_part_picked`
- `has_designator_prefix`
- `has_datasheet_defined`

Generated part definitions map named signals to physical footprint pins:

```ato
signal OUT1 ~ pin 1
signal IN1neg ~ pin 2
signal IN1pos ~ pin 3
```

If manually editing an auto-generated component, remove or update the auto-generated trait as appropriate. Preserve exact manufacturer and MPN fields when they came from the part database.

## Custom Parts

For parts not available through `ato create part`:

1. Create a new folder under the project `parts/` directory named after the component.
2. Add an existing KiCad footprint and 3D model, or create a new footprint in KiCad.
3. Create a `.ato` file in that folder with the same name as the component.
4. Define pins, traits, designator prefix, datasheet, footprint, and model links.

When there is a generic abstract class, subclass it. For example, a concrete LDO should derive from `LDO` and connect concrete pins to the generic LDO signals.

## Packages

Use packages for reusable, tested modules with functioning layout where available.

Find packages:

- https://packages.atopile.io/
- GitHub or other public repositories.

Add a registry package:

```powershell
ato add atopile/ti-ads1115
```

Import from the package:

```ato
from "ti-ads1115.ato" import TI_ADS1115

module ExampleADCProject:
    adc = new TI_ADS1115
```

Package dependencies are recorded in `ato.yaml`, for example:

```yaml
dependencies:
  - type: registry
    identifier: atopile/buttons
    release: 0.2.2
  - type: registry
    identifier: atopile/ti-ads1115
    release: 0.1.6
```

## Dependency Commands

```powershell
ato sync
ato add atopile/ti-ads1115
ato remove atopile/ti-ads1115
```

`ato sync` installs dependencies listed in `ato.yaml`. Run it after pulling a project.

Development dependencies can come from git or local paths:

```powershell
ato add git://{git-url}
ato add file://./path/to/package
```

You cannot publish a package that depends on unpublished packages.
