use bitflags::bitflags;

bitflags! {
    #[derive(Debug, Clone, Copy)]
    pub struct HeaderFlags: u16 {
        /// Header checksum is enabled and must be validated
        const CHECKSUM_ENABLED = 0b0000_0001;

        /// Pages are stored in columnar layout v1
        const COLUMNAR_V1      = 0b0000_0010;

        /// Compression is enabled (algorithm defined elsewhere)
        const COMPRESSION     = 0b0000_0100;

        /// Reserved for future use
        const RESERVED_1      = 0b0000_1000;
    }
}