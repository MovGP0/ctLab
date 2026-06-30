# Compiler And CLI

Sources:

- https://docs.atopile.io/atopile-0.14.x/essentials/2-the-ato-compiler.md
- https://github.com/atopile/atopile

The `ato` command line tool is the main non-editor interface to Atopile.

It can:

- Build code to update a PCB.
- Test a design.
- Generate files for PCB manufacturing.
- Install and manage dependencies.
- Create projects, components, and build targets.

## Command Shape

Atopile follows the common terminal shape:

```text
ato command [options] arguments
```

Use `--help` at any level:

```powershell
ato --help
ato build --help
```

The `-v` verbosity flag is an app-level option, so put it directly after `ato`:

```powershell
ato -v build
```

## Build Flow

`ato build` roughly does this:

1. Finds `ato.yaml`.
2. Compiles code from the configured build entry point.
3. Loads project source and dependencies.
4. Builds the app model of the design.
5. Solves equations and constraints.
6. Picks components, with server recommendations where applicable.
7. Runs tests/checks.
8. Updates the KiCad PCB file.
9. Generates manufacturing data, reports, and configured targets.

## Minimal Example

```ato
import Resistor

module App:
    r1 = new Resistor
    r1.resistance = 50kohm +/- 10%
```

The build entry in `ato.yaml` points to the `App` module, for example `main.ato:App`.

## Better Module Example

Prefer modules with clear external interfaces, documentation, variables, and assertions:

```ato
import Resistor, ElectricPower, ElectricSignal

module VoltageDivider:
    """
    A voltage divider using two resistors.
    """

    power = new ElectricPower
    output = new ElectricSignal

    r_bottom = new Resistor
    r_top = new Resistor

    v_in: voltage
    v_out: voltage
    max_current: current
    r_total: resistance
    ratio: dimensionless

    power.hv ~ r_top.p1
    r_top.p2 ~ output.line
    output.line ~ r_bottom.p1
    r_bottom.p2 ~ power.lv

    assert v_out is output.reference.voltage
    assert v_in is power.voltage
    assert r_total is r_top.resistance + r_bottom.resistance
    assert v_out is v_in * r_bottom.resistance / r_total
    assert max_current is v_in / r_total

module App:
    my_vdiv = new VoltageDivider
    assert my_vdiv.power.voltage is 10V +/- 1%
    assert my_vdiv.output.reference.voltage within 3.3V +/- 10%
    assert my_vdiv.max_current within 10uA to 100uA
```

## Practical Debugging

- If a command fails, rerun with `ato -v ...`.
- If dependencies are missing after pulling a repo, run `ato sync`.
- If KiCad plugin integration is missing, run `ato configure`.
- If a build target is wrong, inspect `ato.yaml` `builds.<name>.entry`, `targets`, `exclude_targets`, and `exclude_checks`.
