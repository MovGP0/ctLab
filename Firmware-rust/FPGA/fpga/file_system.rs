/// Storage boundary used by the controller's SD/FAT command handlers.

///
/// The firmware logic needs whole-file semantics, while the implementation may
/// use a FAT16 card, a simulator, or an in-memory test store.
pub trait FileSystem
{
    /// Backend-specific failure returned without erasing diagnostic detail.
    type Error;

    /// Samples card detection before any operation that would otherwise block or fail obscurely.
    fn card_present(&mut self) -> bool;

    /// Returns root-directory names used by the `DIR`/`LST` protocol commands.
    ///
    /// # Errors
    ///
    /// Returns the backend error when the directory cannot be opened or read.
    fn list_root(&mut self) -> Result<Vec<String>, Self::Error>;

    /// Reads a complete configuration, script, or data file in on-card byte order.
    ///
    /// # Errors
    ///
    /// Returns the backend error for missing files and card or I/O failures.
    fn read_file(&mut self, name: &str) -> Result<Vec<u8>, Self::Error>;

    /// Replaces a file with bytes collected from the FPGA auto-increment port.
    ///
    /// # Errors
    ///
    /// Returns the backend error if creation, truncation, or data writing fails.
    fn write_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>;

    /// Extends log-style files without rewriting their existing content.
    ///
    /// # Errors
    ///
    /// Returns the backend error if the file cannot be opened or extended.
    fn append_file(&mut self, name: &str, data: &[u8]) -> Result<(), Self::Error>;

    /// Removes a directory entry selected by the serial file-management protocol.
    ///
    /// # Errors
    ///
    /// Returns the backend error when the named file is absent or cannot be deleted.
    fn delete_file(&mut self, name: &str) -> Result<(), Self::Error>;
}
