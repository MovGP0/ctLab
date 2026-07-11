use super::*;

#[derive(Debug, Clone)]
pub struct EepromSettings
{
    pub initialized: u16,
    pub serial_baud_register: u8,
    pub hex_mode: bool,
    pub options: [i16; 20],
    pub initial_registers: [i32; 4],
    pub init_file_name: String,
    pub data_file_name: String,
}
impl Default for EepromSettings
{
    fn default() -> Self
    {
        Self
        {
            initialized: EEPROM_INITIALIZED,
            serial_baud_register: 51,
            hex_mode: false,
            options: [
                255, 255, 0, 128, 0, 10, 7, 255, 9, 500, 64, 65, 0, 0, 0, 0, 0, 0, 0, 0,
            ],
            initial_registers: [0; 4],
            init_file_name: DEFAULT_INIT_FILE.to_string(),
            data_file_name: DEFAULT_DATA_FILE.to_string(),
        }
    }
}
