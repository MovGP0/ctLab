//! Type-safe Rust ports of the ctLab AVR firmware families and FPGA controller.
//!
//! Each family retains the state machines and hardware sequencing of its Pascal
//! source while separating device-independent logic from register-level I/O.
//! This lets the same logic run against test doubles and concrete AVR adapters.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// Shared volatile-register primitives and AVR MCU descriptions.
pub mod avrd_support;

#[path = "../DDS/ad9833_control.rs"]
pub mod ad9833_control;
pub use ad9833_control::Ad9833Control;

#[path = "../DDS/waveform.rs"]
pub mod waveform;
pub use waveform::Waveform;

#[path = "../ACV/ACV.rs"]
pub mod acv;

#[path = "../ADA-IO/ADA-C.rs"]
pub mod ada_c;

#[path = "../ADA-IO/ADA-C-HW.rs"]
pub mod ada_c_hw;

#[path = "../ADA-IO/ADA-C-parser.rs"]
pub mod ada_c_parser;

#[path = "../DCG/DCG.rs"]
pub mod dcg;

#[path = "../DCG/DCG-HW.rs"]
pub mod dcg_hw;

#[path = "../DCG/DCG-Parser.rs"]
/// Parses addressed DCG commands and applies their protocol, calibration, and EEPROM side effects.
pub mod dcg_parser;

#[path = "../DDS/DDS-HW.rs"]
pub mod dds_hw;

#[path = "../DDS/DDS.rs"]
pub mod dds;

#[path = "../DDS/DDS-SQG.rs"]
/// Runs the standalone SQG command, panel, burst-timing, and ordered output-programming state machine.
pub mod dds_sqg;

#[path = "../DDS/mp3control.rs"]
pub mod mp3control;

#[path = "../DIV/DIV-HW.rs"]
pub mod div_hw;

#[path = "../DIV/DIV.rs"]
pub mod div;

#[path = "../DIV/DIV-Parser.rs"]
pub mod div_parser;

#[path = "../EDL/EDL-HW.rs"]
pub mod edl_hw;

#[path = "../EDL/EDL.rs"]
pub mod edl;

#[path = "../EDL/EDL-Parser.rs"]
pub mod edl_parser;

#[path = "../FPGA/FPGA-HW.rs"]
pub mod fpga_hw;

#[path = "../FPGA/FPGA-Parser.rs"]
pub mod fpga_parser;

#[path = "../FPGA/FPGA.rs"]
pub mod fpga;
