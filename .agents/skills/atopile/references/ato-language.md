# Ato Language

Source:

- https://docs.atopile.io/atopile-0.14.x/essentials/1-the-ato-language.md

`ato` is a domain-specific language for circuit boards. Its syntax is intentionally Python-like, but the model is electronics-focused.

## Basic Types

| Type | Meaning |
| --- | --- |
| `module` | Reusable design block. Can contain interfaces, components, variables, constraints, and connections. |
| `interface` | A connectable electrical or logical boundary. |
| `Electrical` | Built-in interface representing a single electrical node. |
| `component` | Subclass of `module` expected to represent one physical component. |

You can define and subclass blocks:

```ato
module SomeModule:
    some_signal = new ElectricSignal
    gnd = new Electrical
    some_signal.reference.lv ~ gnd
    some_variable = "some value"

module SubclassedModule from SomeModule:
    some_variable = "some other value"

module Test:
    gnd = new Electrical
    subclassed_module = new SubclassedModule
    subclassed_module.gnd ~ gnd
```

A `module` can be subclassed as a `component`, but a `component` should not be subclassed back into a generic module because it represents a specific physical component expectation.

## Configuration

Configure modules, interfaces, and components by assigning to attributes:

```ato
some_instance.value = 100ohm +/- 10%
```

Inside a block, assignments are automatically assigned to that block. You do not write `self.`.

Some attributes affect compiler behavior. For example, setting `package` constrains component selection to a package:

```ato
resistor = new Resistor
resistor.resistance = 10kohm +/- 5%
resistor.package = "0402"
```

## Connections

Connect interfaces of the same type with `~`:

```ato
some_signal ~ another
```

Connections are core circuit topology. Preserve type-compatible interfaces and do not replace typed interfaces with raw pins unless the task is explicitly about low-level part definition.

## Units And Tolerances

Physical values should include units. Examples:

```ato
r.resistance = 10kohm +/- 1%
c.capacitance = 4.7uF +/- 20%
rail.voltage = 3V to 3.6V
reference.voltage = 3V +/- 10mV
```

Tolerance forms:

- `1V to 2V`
- `3uF +/- 1uF`
- `4Kohm +/- 1%`

Use units consistently. The compiler uses units and tolerances when checking constraints and solving for component values.

## Assertions And Constraints

Use `assert` to describe required relationships.

Supported comparison-style operators include:

- `<`
- `>`
- `within`

The docs also show `is` for equations and identity-like constraint relationships.

```ato
a = 1 +/- 0.1
b = 2 +/- 0.2
c: resistance

assert a < b
assert c within 1Kohm to 10Kohm
```

The compiler can solve systems of constraints for free variables and then check that solved values fall within their tolerances.

## Specialization

Use `->` to specialize an existing module instance to another type:

```ato
some_instance -> AnotherModuleType
```

This is useful when a topology is specified earlier and a later layer chooses a concrete implementation.

## Imports

Use the current import syntax:

```ato
from "where.ato" import What, Why, Wow
```

Rules:

- Quote the source path.
- Imported names must match exactly.
- Type names are typically capitalized.
- Imports are relative to the project root containing `ato.yaml`, or within the standard library under `.ato/modules/`.

Avoid the legacy form:

```ato
import XYZ from "abc.ato"
```

The docs warn that the legacy syntax will be removed and does not support importing multiple things on one line.
