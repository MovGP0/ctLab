# PCB Trace Width

Trace width calculations use IPC-2221 curve-fit formulas for PCB conductor area and then derive trace width, resistance, voltage drop, and power loss for internal and external layers.

## Symbols

| Symbol | Meaning | Unit |
| --- | --- | --- |
| `I` | Current | A |
| `Delta T` | Temperature rise | degrees C |
| `Ta` | Ambient temperature | degrees C |
| `Tc` | Conductor temperature | degrees C |
| `A` | Required conductor cross-sectional area | mil^2 |
| `Acm2` | Required conductor cross-sectional area | cm^2 |
| `t` | Copper thickness | cm |
| `W` | Required trace width | cm |
| `L` | Trace length | cm |
| `R` | Trace resistance | Ohms |
| `V` | Voltage drop | V |
| `P` | Power loss | W |
| `rho25` | Copper resistivity at 25 degrees C | Ohm cm |
| `alpha` | Copper temperature coefficient | 1/degrees C |

## IPC-2221 Constants

```text
b = 0.44
c = 0.725
k_internal = 0.024
k_external = 0.048
```

Use `k_internal` for internal layers and `k_external` for external layers in air.

## Temperature

Convert temperature rise from Fahrenheit:

```text
DeltaT_C = DeltaT_F * 5 / 9
```

Convert ambient temperature from Fahrenheit:

```text
Ta_C = (Ta_F - 32) * 5 / 9
```

The conductor temperature used for resistance is:

```text
Tc = Ta + DeltaT
```

## Required Area

For either layer type, using the matching IPC-2221 constant `k`:

```text
A = (I / (k * DeltaT^b))^(1 / c)
```

The inverse current formula is:

```text
I = k * DeltaT^b * A^c
```

Convert `mil^2` to `cm^2`:

```text
Acm2 = A * 2.54^2 / 1000000
Acm2 = A * 6.4516e-6
```

## Copper Thickness

Convert copper thickness to centimeters before calculating trace width:

```text
t_cm = x * unit_factor
```

| Input unit | Factor |
| --- | ---: |
| oz/ft^2 | 0.0035 |
| mil | 2.54e-3 |
| mm | 0.1 |
| um | 1e-4 |

## Trace Width

In centimeters:

```text
W_cm = Acm2 / t_cm
```

For output in another width unit:

```text
W_out = W_cm / width_unit_factor
```

| Output unit | Factor |
| --- | ---: |
| mil | 2.54e-3 |
| mm | 0.1 |
| um | 1e-4 |

For copper weight in ounces, the direct mil formula is:

```text
W_mil = A_mil2 / (t_oz * 1.378)
```

## Trace Length

Convert entered length to centimeters:

```text
L_cm = x / length_unit_factor
```

| Input unit | Factor |
| --- | ---: |
| inch | 0.393701 |
| feet | 0.032808 |
| mil | 393.7008 |
| mm | 10 |
| um | 10000 |
| cm | 1 |
| m | 0.01 |

## Resistance, Voltage Drop, And Power Loss

For copper:

```text
rho25 = 17e-7 Ohm cm
alpha = 0.0039 1/degrees C
```

Temperature-adjusted resistance:

```text
R = (rho25 * L_cm / Acm2) * (1 + alpha * (Tc - 25))
```

Voltage drop:

```text
V = I * R
```

Power loss:

```text
P = I^2 * R
```

## Validity Notes

The original IPC-2221 graphs cover approximately:

- Current up to 35 A.
- Width up to 0.4 inch.
- Temperature rise from 10 degrees C to 100 degrees C.
- Copper from 0.5 to 3 oz/ft^2.

Outside these ranges, results are extrapolated.

## Source

- https://www.advancedpcb.com/en-us/tools/trace-width-calculator/
