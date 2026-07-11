/// Identifies the FPGA controller's textual commands without relying on parallel string tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdWhich
{
    /// `STR` — Status: reports packed runtime, card, and error flags on subchannel 255.
    Str,

    /// `IDN` — Identification: returns the FPGA firmware name and version on subchannel 254.
    Idn,

    /// `VAL` — Value: addresses a numeric subchannel directly so scripts can use one generic mnemonic.
    Val,

    /// `REG` — Register: reads or writes one of ten floating-point script registers at 300 through 309.
    Reg,

    /// `ACC` — Accumulator: aliases register zero because arithmetic instructions use it as accumulator A.
    Acc,

    /// `MOV` — Move: copies the register selected by the value into the register selected by the suffix.
    Mov,

    /// `DEC` — Decrement: subtracts one from a selected register and retains the result for branching.
    Dec,

    /// `INC` — Increment: adds one to a selected register and retains the result for branching.
    Inc,

    /// `CPZ` — Compare Zero: compares a selected register with zero so a later branch can test the result.
    Cpz,

    /// `XCH` — Exchange: swaps accumulator A with a selected register for compact script calculations.
    Xch,

    /// `GET` — Get: requests a remote subchannel and stores its eventual result in a selected register.
    Get,

    /// `PUT` — Put: sends accumulator A to the configured remote main channel and subchannel.
    Put,

    /// `MUL` — Multiply: multiplies accumulator A by a selected register.
    Mul,

    /// `DIV` — Divide: divides accumulator A by a selected register and rejects a zero divisor.
    Div,

    /// `ADD` — Add: adds a selected register to accumulator A.
    Add,

    /// `SUB` — Subtract: subtracts a selected register from accumulator A.
    Sub,

    /// `SQR` — Square Root: replaces a selected register with its square root.
    Sqr,

    /// `SQU` — Square: multiplies a selected register by itself.
    Squ,

    /// `NEG` — Negate: reverses the sign of a selected register.
    Neg,

    /// `LBL` — Label: records a script file position for a label numbered 0 through 31.
    Lbl,

    /// `GTO` — Go To: jumps unconditionally to a script label and shares the branch-always subchannel.
    Gto,

    /// `BRA` — Branch Always: jumps unconditionally to a script label and aliases `GTO`.
    Bra,

    /// `BRG` — Branch Greater: jumps when the last comparison result is greater than zero.
    Brg,

    /// `BGE` — Branch Greater or Equal: jumps when the last comparison is non-negative.
    Bge,

    /// `BEQ` — Branch Equal: jumps when the last comparison result is zero.
    Beq,

    /// `BLE` — Branch Less or Equal: jumps when the last comparison is non-positive.
    Ble,

    /// `BRL` — Branch Less: jumps when the last comparison result is less than zero.
    Brl,

    /// `INP` — Input: exchanges an FPGA register and places its received value in accumulator A.
    Inp,

    /// `OUT` — Output: sends an accumulator or script-register value to an FPGA register.
    Out,

    /// `TTF` — Type Text File: streams a card file byte-by-byte to a selected FPGA SPI register.
    Ttf,

    /// `TTY` — Teletype: streams a card file to serial output and expands carriage returns for terminals.
    Tty,

    /// `TSF` — Text Save File: appends incoming serial text to a card file until an ETX character arrives.
    Tsf,

    /// `XMR` — XMODEM Receive: receives a checksum-framed block stream into auto-increment FPGA memory.
    Xmr,

    /// `TSR` — Type String to Register: sends text through a selected internal FPGA serial register.
    Tsr,

    /// `TSS` — Type String Serial: writes text followed by CR/LF to the external serial route.
    Tss,

    /// `COM` — Communication: configures internal RX/TX registers and the script response route.
    Com,

    /// `AIR` — Auto-Increment Register: selects the FPGA data register used for block transfers.
    Air,

    /// `AIS` — Auto-Increment Select: selects the FPGA memory bank or core before a block transfer.
    Ais,

    /// `AIW` — Auto-Increment Width: chooses one-, two-, or four-byte file elements for block transfers.
    Aiw,

    /// `BLD` — Block Load: reads a binary card file and streams it into auto-increment FPGA memory.
    Bld,

    /// `BSV` — Block Save: reads an FPGA address range and stores it in a binary card file.
    Bsv,

    /// `AIM` — Auto-Increment Minimum: sets the first FPGA address used by block transfers.
    Aim,

    /// `AIE` — Auto-Increment End: sets the exclusive final FPGA address used when saving a block.
    Aie,

    /// `CLK` — Clock: accesses the real-time clock fields and commands used by timed scripts.
    Clk,

    /// `OPT` — Option: accesses EEPROM-backed startup files, register defaults, and routing settings.
    Opt,

    /// `MCH` — Main Channel: selects the destination module address used by script `GET` and `PUT`.
    Mch,

    /// `SCH` — Subchannel: selects the destination subchannel used by script `GET` and `PUT`.
    Sch,

    /// `WTH` — Wait Tick Hour: pauses script execution until the next real-time-clock hour tick.
    Wth,

    /// `WTM` — Wait Tick Minute: pauses script execution until the next real-time-clock minute tick.
    Wtm,

    /// `WTS` — Wait Tick Second: pauses script execution until the next real-time-clock second tick.
    Wts,

    /// `DLY` — Delay: pauses script execution for the supplied number of milliseconds.
    Dly,

    /// `FWR` — File Write Register: appends a selected script register and transfer metadata to the data file.
    Fwr,

    /// `FWV` — File Write Value: appends a supplied value and index with time and date to the data file.
    Fwv,

    /// `CFG` — Configuration: loads a named or indexed configuration, script, data, or FPGA file.
    Cfg,

    /// `LST` — List: refreshes and emits the card's root-directory entries.
    Lst,

    /// `DIR` — Directory: aliases `LST` so either mnemonic emits the card directory.
    Dir,

    /// `FNM` — File Number: reports the number of entries in the bounded directory cache.
    Fnm,

    /// `FNA` — File Name: reads or selects the data file used by later write commands.
    Fna,

    /// `FDL` — File Delete: removes a named or indexed card file and refreshes the directory cache.
    Fdl,

    /// `FQU` — File Query: reports whether a named or indexed file exists on the card.
    Fqu,

    /// `HEX` — Hexadecimal: selects hexadecimal numeric formatting for serial responses.
    Hex,

    /// `WEN` — Write Enable: unlocks one protected EEPROM write to prevent accidental persistence changes.
    Wen,

    /// `ERC` — Error Count: reads or resets the number of parser errors accumulated since reset.
    Erc,

    /// `SBD` — Serial Baud Divisor: reads or stores the AVR UART divisor applied after the next reset.
    Sbd,

    /// `REM` — Remark: maps script comments to the inert subchannel so explanatory lines have no side effects.
    Rem,

    /// `NOP` — No Operation: validates framing without changing controller state.
    Nop,

    /// Internal sentinel returned when no FPGA command mnemonic matches.
    Err,
}

impl CmdWhich
{
    /// Returns the exact three-letter FPGA wire mnemonic, or `None` for the internal error sentinel.
    pub const fn as_str(self) -> Option<&'static str>
    {
        match self
        {
            Self::Str => Some("STR"),
            Self::Idn => Some("IDN"),
            Self::Val => Some("VAL"),
            Self::Reg => Some("REG"),
            Self::Acc => Some("ACC"),
            Self::Mov => Some("MOV"),
            Self::Dec => Some("DEC"),
            Self::Inc => Some("INC"),
            Self::Cpz => Some("CPZ"),
            Self::Xch => Some("XCH"),
            Self::Get => Some("GET"),
            Self::Put => Some("PUT"),
            Self::Mul => Some("MUL"),
            Self::Div => Some("DIV"),
            Self::Add => Some("ADD"),
            Self::Sub => Some("SUB"),
            Self::Sqr => Some("SQR"),
            Self::Squ => Some("SQU"),
            Self::Neg => Some("NEG"),
            Self::Lbl => Some("LBL"),
            Self::Gto => Some("GTO"),
            Self::Bra => Some("BRA"),
            Self::Brg => Some("BRG"),
            Self::Bge => Some("BGE"),
            Self::Beq => Some("BEQ"),
            Self::Ble => Some("BLE"),
            Self::Brl => Some("BRL"),
            Self::Inp => Some("INP"),
            Self::Out => Some("OUT"),
            Self::Ttf => Some("TTF"),
            Self::Tty => Some("TTY"),
            Self::Tsf => Some("TSF"),
            Self::Xmr => Some("XMR"),
            Self::Tsr => Some("TSR"),
            Self::Tss => Some("TSS"),
            Self::Com => Some("COM"),
            Self::Air => Some("AIR"),
            Self::Ais => Some("AIS"),
            Self::Aiw => Some("AIW"),
            Self::Bld => Some("BLD"),
            Self::Bsv => Some("BSV"),
            Self::Aim => Some("AIM"),
            Self::Aie => Some("AIE"),
            Self::Clk => Some("CLK"),
            Self::Opt => Some("OPT"),
            Self::Mch => Some("MCH"),
            Self::Sch => Some("SCH"),
            Self::Wth => Some("WTH"),
            Self::Wtm => Some("WTM"),
            Self::Wts => Some("WTS"),
            Self::Dly => Some("DLY"),
            Self::Fwr => Some("FWR"),
            Self::Fwv => Some("FWV"),
            Self::Cfg => Some("CFG"),
            Self::Lst => Some("LST"),
            Self::Dir => Some("DIR"),
            Self::Fnm => Some("FNM"),
            Self::Fna => Some("FNA"),
            Self::Fdl => Some("FDL"),
            Self::Fqu => Some("FQU"),
            Self::Hex => Some("HEX"),
            Self::Wen => Some("WEN"),
            Self::Erc => Some("ERC"),
            Self::Sbd => Some("SBD"),
            Self::Rem => Some("REM"),
            Self::Nop => Some("NOP"),
            Self::Err => None,
        }
    }

    /// Parses an FPGA mnemonic without allocation while accepting ASCII case and surrounding whitespace.
    pub fn from_mnemonic(keyword: &str) -> Self
    {
        let keyword = keyword.trim();
        if keyword.eq_ignore_ascii_case("STR") { Self::Str }
        else if keyword.eq_ignore_ascii_case("IDN") { Self::Idn }
        else if keyword.eq_ignore_ascii_case("VAL") { Self::Val }
        else if keyword.eq_ignore_ascii_case("REG") { Self::Reg }
        else if keyword.eq_ignore_ascii_case("ACC") { Self::Acc }
        else if keyword.eq_ignore_ascii_case("MOV") { Self::Mov }
        else if keyword.eq_ignore_ascii_case("DEC") { Self::Dec }
        else if keyword.eq_ignore_ascii_case("INC") { Self::Inc }
        else if keyword.eq_ignore_ascii_case("CPZ") { Self::Cpz }
        else if keyword.eq_ignore_ascii_case("XCH") { Self::Xch }
        else if keyword.eq_ignore_ascii_case("GET") { Self::Get }
        else if keyword.eq_ignore_ascii_case("PUT") { Self::Put }
        else if keyword.eq_ignore_ascii_case("MUL") { Self::Mul }
        else if keyword.eq_ignore_ascii_case("DIV") { Self::Div }
        else if keyword.eq_ignore_ascii_case("ADD") { Self::Add }
        else if keyword.eq_ignore_ascii_case("SUB") { Self::Sub }
        else if keyword.eq_ignore_ascii_case("SQR") { Self::Sqr }
        else if keyword.eq_ignore_ascii_case("SQU") { Self::Squ }
        else if keyword.eq_ignore_ascii_case("NEG") { Self::Neg }
        else if keyword.eq_ignore_ascii_case("LBL") { Self::Lbl }
        else if keyword.eq_ignore_ascii_case("GTO") { Self::Gto }
        else if keyword.eq_ignore_ascii_case("BRA") { Self::Bra }
        else if keyword.eq_ignore_ascii_case("BRG") { Self::Brg }
        else if keyword.eq_ignore_ascii_case("BGE") { Self::Bge }
        else if keyword.eq_ignore_ascii_case("BEQ") { Self::Beq }
        else if keyword.eq_ignore_ascii_case("BLE") { Self::Ble }
        else if keyword.eq_ignore_ascii_case("BRL") { Self::Brl }
        else if keyword.eq_ignore_ascii_case("INP") { Self::Inp }
        else if keyword.eq_ignore_ascii_case("OUT") { Self::Out }
        else if keyword.eq_ignore_ascii_case("TTF") { Self::Ttf }
        else if keyword.eq_ignore_ascii_case("TTY") { Self::Tty }
        else if keyword.eq_ignore_ascii_case("TSF") { Self::Tsf }
        else if keyword.eq_ignore_ascii_case("XMR") { Self::Xmr }
        else if keyword.eq_ignore_ascii_case("TSR") { Self::Tsr }
        else if keyword.eq_ignore_ascii_case("TSS") { Self::Tss }
        else if keyword.eq_ignore_ascii_case("COM") { Self::Com }
        else if keyword.eq_ignore_ascii_case("AIR") { Self::Air }
        else if keyword.eq_ignore_ascii_case("AIS") { Self::Ais }
        else if keyword.eq_ignore_ascii_case("AIW") { Self::Aiw }
        else if keyword.eq_ignore_ascii_case("BLD") { Self::Bld }
        else if keyword.eq_ignore_ascii_case("BSV") { Self::Bsv }
        else if keyword.eq_ignore_ascii_case("AIM") { Self::Aim }
        else if keyword.eq_ignore_ascii_case("AIE") { Self::Aie }
        else if keyword.eq_ignore_ascii_case("CLK") { Self::Clk }
        else if keyword.eq_ignore_ascii_case("OPT") { Self::Opt }
        else if keyword.eq_ignore_ascii_case("MCH") { Self::Mch }
        else if keyword.eq_ignore_ascii_case("SCH") { Self::Sch }
        else if keyword.eq_ignore_ascii_case("WTH") { Self::Wth }
        else if keyword.eq_ignore_ascii_case("WTM") { Self::Wtm }
        else if keyword.eq_ignore_ascii_case("WTS") { Self::Wts }
        else if keyword.eq_ignore_ascii_case("DLY") { Self::Dly }
        else if keyword.eq_ignore_ascii_case("FWR") { Self::Fwr }
        else if keyword.eq_ignore_ascii_case("FWV") { Self::Fwv }
        else if keyword.eq_ignore_ascii_case("CFG") { Self::Cfg }
        else if keyword.eq_ignore_ascii_case("LST") { Self::Lst }
        else if keyword.eq_ignore_ascii_case("DIR") { Self::Dir }
        else if keyword.eq_ignore_ascii_case("FNM") { Self::Fnm }
        else if keyword.eq_ignore_ascii_case("FNA") { Self::Fna }
        else if keyword.eq_ignore_ascii_case("FDL") { Self::Fdl }
        else if keyword.eq_ignore_ascii_case("FQU") { Self::Fqu }
        else if keyword.eq_ignore_ascii_case("HEX") { Self::Hex }
        else if keyword.eq_ignore_ascii_case("WEN") { Self::Wen }
        else if keyword.eq_ignore_ascii_case("ERC") { Self::Erc }
        else if keyword.eq_ignore_ascii_case("SBD") { Self::Sbd }
        else if keyword.eq_ignore_ascii_case("REM") { Self::Rem }
        else if keyword.eq_ignore_ascii_case("NOP") { Self::Nop }
        else { Self::Err }
    }

    /// Returns the Pascal base subchannel before the command's optional numeric suffix is added.
    pub const fn sub_channel_offset(self) -> Option<u16>
    {
        match self
        {
            Self::Str => Some(255),
            Self::Idn => Some(254),
            Self::Val => Some(0),
            Self::Reg | Self::Acc => Some(300),
            Self::Mov => Some(310),
            Self::Dec => Some(320),
            Self::Inc => Some(330),
            Self::Cpz => Some(340),
            Self::Xch => Some(350),
            Self::Get => Some(400),
            Self::Put => Some(500),
            Self::Mul => Some(600),
            Self::Div => Some(610),
            Self::Add => Some(620),
            Self::Sub => Some(630),
            Self::Sqr => Some(640),
            Self::Squ => Some(650),
            Self::Neg => Some(660),
            Self::Lbl => Some(1000),
            Self::Gto | Self::Bra => Some(1100),
            Self::Brg => Some(1200),
            Self::Bge => Some(1300),
            Self::Beq => Some(1400),
            Self::Ble => Some(1500),
            Self::Brl => Some(1600),
            Self::Inp | Self::Out => Some(2000),
            Self::Ttf => Some(800),
            Self::Tty => Some(880),
            Self::Tsf => Some(881),
            Self::Xmr => Some(890),
            Self::Tsr => Some(900),
            Self::Tss => Some(980),
            Self::Com => Some(990),
            Self::Air => Some(280),
            Self::Ais => Some(281),
            Self::Aiw => Some(282),
            Self::Bld => Some(283),
            Self::Bsv => Some(284),
            Self::Aim => Some(285),
            Self::Aie => Some(286),
            Self::Clk => Some(90),
            Self::Opt => Some(150),
            Self::Mch => Some(270),
            Self::Sch => Some(271),
            Self::Wth => Some(290),
            Self::Wtm => Some(291),
            Self::Wts => Some(292),
            Self::Dly => Some(299),
            Self::Fwr => Some(220),
            Self::Fwv => Some(230),
            Self::Cfg => Some(240),
            Self::Lst | Self::Dir => Some(241),
            Self::Fnm => Some(242),
            Self::Fna => Some(243),
            Self::Fdl => Some(244),
            Self::Fqu => Some(249),
            Self::Hex => Some(88),
            Self::Wen => Some(250),
            Self::Erc => Some(251),
            Self::Sbd => Some(252),
            Self::Rem | Self::Nop => Some(253),
            Self::Err => None,
        }
    }

    /// Adds the optional numeric suffix while rejecting unknown commands and `u16` overflow.
    pub const fn sub_channel(self, argument: u16) -> Option<u16>
    {
        match self.sub_channel_offset()
        {
            Some(offset) => offset.checked_add(argument),
            None => None,
        }
    }
}
