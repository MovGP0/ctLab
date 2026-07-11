pub trait FileSystem
{
    type Error;

    fn card_present(&mut self) -> bool;

    fn list_root(&mut self) -> Result<Vec<String>, Self::Error>;

    fn read_file(&mut self, name: &str) -> Result<Vec<u8>, Self::Error>;

    fn write_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>;

    fn append_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>;

    fn delete_file(&mut self, name: &str) -> Result<(), Self::Error>;
}
