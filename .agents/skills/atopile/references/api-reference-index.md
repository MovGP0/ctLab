# API Reference Index

Source:

- https://docs.atopile.io/llms.txt

Use this file to choose the likely standard-library component or interface name before opening the full official API page.

## Common Components

| Component | Purpose or note |
| --- | --- |
| `Resistor` | Generic resistor. |
| `Capacitor` | Generic capacitor. |
| `Inductor` | Generic inductor. |
| `Diode` | Generic diode. |
| `LED` | LED component. |
| `LEDIndicator` | LED indicator module. |
| `MOSFET`, `NFET`, `PFET` | FET components. |
| `BJT` | Bipolar transistor. |
| `OpAmp` | Operational amplifier. |
| `Comparator` | Comparator. |
| `LDO` | Low-dropout regulator abstraction. |
| `Regulator` | Generic regulator. |
| `PowerMux` | Power mux. |
| `PowerSwitch`, `PowerSwitchMOSFET`, `PowerSwitchStatic` | Power switching modules. |
| `Fuse` | Fuse. |
| `TVS`, `SurgeProtection`, `GDT` | Protection devices. |
| `Crystal`, `Crystal_Oscillator` | Timing components. |
| `EEPROM`, `SPIFlash` | Memory components. |
| `FilterElectricalRC`, `FilterElectricalLC` | Basic RC/LC filter modules. |
| `ResistorVoltageDivider` | Voltage divider using two resistors. |
| `TestPoint` | Basic test point. |
| `Footprint`, `KicadFootprint`, `Symbol` | KiCad/layout-related components. |

## Common Interfaces

| Interface | Purpose or note |
| --- | --- |
| `Electrical` | Single electrical node. |
| `ElectricPower` | Power rail with `hv` and `lv`. |
| `ElectricSignal` | Signal represented by voltage between a reference high/low. |
| `ElectricLogic` | Logic signal with high and low states. |
| `DifferentialPair` | Differential pair. |
| `I2C`, `SPI`, `MultiSPI`, `UART`, `CAN`, `RS232`, `RS485HalfDuplex` | Serial interfaces. |
| `USB2_0`, `USB2_0_IF`, `USB3`, `USB3_IF`, `USB_C` | USB interfaces. |
| `Ethernet` | 1000BASE-T Gigabit Ethernet interface. |
| `HDMI` | HDMI interface. |
| `I2S`, `PDM` | Digital audio interfaces. |
| `JTAG`, `SWD` | Debug/programming interfaces. |
| `EnablePin` | Enable pin abstraction. |
| `Pad`, `Mechanical` | Pad and mechanical interfaces. |

## Traits Mentioned In Docs

Generated and custom part definitions commonly use traits such as:

- `is_auto_generated`
- `is_atomic_part`
- `has_part_picked`
- `has_designator_prefix`
- `has_datasheet_defined`
- `can_bridge_by_name`
- `requires_pulls`

Open the current official page for exact trait arguments before changing trait-heavy component definitions.
