# Simulation

NGSpice is the SPICE simulator used by KiCad's simulation workflow.

## Windows

Chocolatey is a quick install option:

```powershell
choco install ngspice -y
ngspice -v
```

The Chocolatey package may lag behind the latest upstream NGSpice release. For the latest Windows binary, use the official downloads:

- https://ngspice.sourceforge.io/download.html
- https://ngspice.sourceforge.io/packages.html

After extracting or installing NGSpice, add the folder containing `ngspice.exe` to `PATH`, then verify it from PowerShell:

```powershell
ngspice -v
```

If KiCad does not find NGSpice automatically, configure the simulator path in KiCad's preferences or run simulations from a terminal where `ngspice.exe` is already on `PATH`.

## WSL / Ubuntu

```bash
sudo apt update
sudo apt install ngspice
ngspice -v
```

## macOS

```bash
brew install ngspice
ngspice -v
```

## Schematic Properties

For KiCad simulation symbols, preserve and review these properties:

| Property | Purpose |
| --- | --- |
| `Sim.Device` | SPICE device kind used by simulation symbols |
| `Sim.Type` | SPICE model or primitive type used by simulation symbols |
| `Sim.Pins` | Mapping between KiCad pins and SPICE pins |

For CMOS transistor symbols in the local notes, use:

```text
Sim.Pins = "1=D 2=G 3=S"
```

Use `Simulation_SPICE:0` for the SPICE 0 V reference node when the simulation requires an explicit ground reference.
