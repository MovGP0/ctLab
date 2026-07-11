#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProtectionFlags {
    pub over_power: bool,
    pub fuse_blown: bool,
    pub over_voltage: bool,
    pub over_temp: bool,
    pub low_volt: bool,
}

impl ProtectionFlags {
    pub fn any(self) -> bool {
        self.bits() != 0
    }

    pub fn bits(self) -> u8 {
        (self.over_power as u8)
            | ((self.fuse_blown as u8) << 1)
            | ((self.over_voltage as u8) << 2)
            | ((self.over_temp as u8) << 3)
            | ((self.low_volt as u8) << 4)
    }
}
