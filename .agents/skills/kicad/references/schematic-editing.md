# Schematic Editing

## Grid Alignment

- KiCad schematic coordinates are stored in millimeters, not pixels or arbitrary integer grid units.
- The local CMOS schematic notes use a `1.27 mm` grid, which is KiCad's `50 mil` schematic grid.
- Do not judge grid alignment by whether the decimal representation looks round. A coordinate such as `160.02` is on the `1.27 mm` grid because `126 * 1.27 = 160.02`.
- Wires, junctions, connector symbols, power symbols, labels, and transistor symbol origins should be placed on the same grid.
- Aligning only wire endpoints is not enough. Component origins must also be positioned so transformed pin endpoints land on the wire grid.
- Embedded `lib_symbols` geometry can use smaller drawing coordinates and should not be treated as schematic placement.

## Pins And Connection Points

Pins are electrical connection points of a symbol. A wire connects to the pin's `(at X Y ANGLE)` coordinate after the symbol placement transform is applied.

For an unrotated placed symbol at `(at X Y 0)`, local pin coordinates are added to the placed origin:

| Example pin | Local connection point | Placed connection point |
| --- | --- | --- |
| NMOS drain `(at 2.54 5.08 270)` | `(2.54, 5.08)` | `(X + 2.54, Y + 5.08)` |
| NMOS gate `(at -5.08 0 0)` | `(-5.08, 0)` | `(X - 5.08, Y)` |
| NMOS source `(at 2.54 -5.08 90)` | `(2.54, -5.08)` | `(X + 2.54, Y - 5.08)` |
| Connector pin `(at -5.08 0 0)` | `(-5.08, 0)` | `(X - 5.08, Y)` |

Rotation and mirroring change these transformed connection points. Check the final placed symbol transform before routing wires.

## CMOS Local Style

The local inverter schematic style uses:

- Connector symbols for external pins.
- Power symbols for `VDD` and `GND`.
- Transistor symbols with `Sim.Device`, `Sim.Type`, and `Sim.Pins` properties for SPICE export.
- `Sim.Pins = "1=D 2=G 3=S"` for CMOS transistor symbols.
- Input connectors on the left, output connectors on the right, PMOS above NMOS, `VDD` at the top, and `GND` at the bottom.

For unrotated `Simulation_SPICE:PMOS` and `Simulation_SPICE:NMOS` symbols, pin endpoints are relative to the symbol origin:

| Pin | Endpoint |
| --- | --- |
| Drain | `(origin_x + 2.54, origin_y + 5.08)` |
| Gate | `(origin_x - 5.08, origin_y)` |
| Source | `(origin_x + 2.54, origin_y - 5.08)` |

## Power Symbols

A placed power symbol needs:

- A matching `lib_symbols` entry.
- A schematic-level `symbol` instance.
- A `lib_id` that references the library symbol.
- A `Value` property that defines the global net name created by the power symbol.

Typical power references use hidden references such as `#PWR01`. `PWR_FLAG` is an ERC marker used to declare that a net is driven by a power source.

Common power nets include `power:GND`, `power:VDD`, `power:VCC`, `power:VSS`, and simulation `Simulation_SPICE:0`.

## Reference Designator Edits

When systematically renaming visible references in `.kicad_sch` files:

- Rewrite only schematic-level placed symbol references unless the symbol definition itself truly changes.
- Update `property "Reference"` and matching instance `(reference "...")` values together.
- Leave `lib_id` alone unless the component library identity changes.
- Preserve KiCad power reference conventions such as `#PWR` and `#FLG` unless the task explicitly asks to rename them.
