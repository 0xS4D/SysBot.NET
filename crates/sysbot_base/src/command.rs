//! Switch command encoding and decoding
//!
//! This module handles encoding commands for sys-botbase and decoding responses.
//! All commands use the sys-botbase protocol which expects ASCII text commands
//! terminated with CRLF (\r\n).
//!
//! # Protocol Reference
//!
//! ## Button Commands
//! - `click {BUTTON}\r\n` - Press and release a button
//! - `press {BUTTON}\r\n` - Press and hold a button
//! - `release {BUTTON}\r\n` - Release a held button
//!
//! ## Stick Commands
//! - `setStick {STICK} {X} {Y}\r\n` - Set stick position (-32768 to 32767)
//!
//! ## Memory Commands
//! - `peek 0x{OFFSET} {SIZE}\r\n` - Read from heap
//! - `poke 0x{OFFSET} 0x{DATA}\r\n` - Write to heap
//! - `peekMain 0x{OFFSET} {SIZE}\r\n` - Read from main
//! - `pokeMain 0x{OFFSET} 0x{DATA}\r\n` - Write to main
//! - `peekAbsolute 0x{OFFSET} {SIZE}\r\n` - Read from absolute address
//! - `pokeAbsolute 0x{OFFSET} 0x{DATA}\r\n` - Write to absolute address
//!
//! ## Pointer Commands
//! - `pointerPeek [0x{OFFSET}]+ {SIZE}\r\n` - Follow pointer chain and read
//! - `pointerPoke [0x{OFFSET}]+ 0x{DATA}\r\n` - Follow pointer chain and write
//!
//! ## System Commands
//! - `getMainNsoBase\r\n` - Get main NSO base address
//! - `getHeapBase\r\n` - Get heap base address
//! - `getTitleID\r\n` - Get current game's title ID
//! - `detachController\r\n` - Detach virtual controller
//! - `configure {PARAM} {VALUE}\r\n` - Configure sys-botbase settings

use std::fmt;

/// Nintendo Switch controller buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchButton {
    A,
    B,
    X,
    Y,
    L,
    R,
    ZL,
    ZR,
    Plus,
    Minus,
    DUp,
    DDown,
    DLeft,
    DRight,
    LStick,
    RStick,
    Home,
    Capture,
}

impl fmt::Display for SwitchButton {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::A => "A",
            Self::B => "B",
            Self::X => "X",
            Self::Y => "Y",
            Self::L => "L",
            Self::R => "R",
            Self::ZL => "ZL",
            Self::ZR => "ZR",
            Self::Plus => "PLUS",
            Self::Minus => "MINUS",
            Self::DUp => "DUP",
            Self::DDown => "DDOWN",
            Self::DLeft => "DLEFT",
            Self::DRight => "DRIGHT",
            Self::LStick => "LSTICK",
            Self::RStick => "RSTICK",
            Self::Home => "HOME",
            Self::Capture => "CAPTURE",
        };
        write!(f, "{}", name)
    }
}

/// Nintendo Switch analog sticks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SwitchStick {
    Left,
    Right,
}

impl fmt::Display for SwitchStick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => write!(f, "LEFT"),
            Self::Right => write!(f, "RIGHT"),
        }
    }
}

/// Memory offset types for read/write operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffsetType {
    /// Relative to heap base
    Heap,
    /// Relative to main NSO base
    Main,
    /// Absolute memory address
    Absolute,
}

/// Builder for sys-botbase commands
pub struct SwitchCommand;

impl SwitchCommand {
    /// Click a button (press and release)
    pub fn click(button: SwitchButton) -> Vec<u8> {
        format!("click {}\r\n", button).into_bytes()
    }

    /// Press and hold a button
    pub fn press(button: SwitchButton) -> Vec<u8> {
        format!("press {}\r\n", button).into_bytes()
    }

    /// Release a held button
    pub fn release(button: SwitchButton) -> Vec<u8> {
        format!("release {}\r\n", button).into_bytes()
    }

    /// Set analog stick position
    /// X and Y range from -32768 to 32767
    pub fn set_stick(stick: SwitchStick, x: i16, y: i16) -> Vec<u8> {
        format!("setStick {} {} {}\r\n", stick, x, y).into_bytes()
    }

    /// Reset stick to center position
    pub fn reset_stick(stick: SwitchStick) -> Vec<u8> {
        Self::set_stick(stick, 0, 0)
    }

    /// Read memory from the specified offset type
    pub fn peek(offset: u64, size: usize, offset_type: OffsetType) -> Vec<u8> {
        let cmd = match offset_type {
            OffsetType::Heap => "peek",
            OffsetType::Main => "peekMain",
            OffsetType::Absolute => "peekAbsolute",
        };
        format!("{} 0x{:X} {}\r\n", cmd, offset, size).into_bytes()
    }

    /// Write memory to the specified offset type
    pub fn poke(offset: u64, data: &[u8], offset_type: OffsetType) -> Vec<u8> {
        let cmd = match offset_type {
            OffsetType::Heap => "poke",
            OffsetType::Main => "pokeMain",
            OffsetType::Absolute => "pokeAbsolute",
        };
        let hex_data = data.iter().map(|b| format!("{:02X}", b)).collect::<String>();
        format!("{} 0x{:X} 0x{}\r\n", cmd, offset, hex_data).into_bytes()
    }

    /// Follow a pointer chain and read memory
    pub fn pointer_peek(offsets: &[i64], size: usize) -> Vec<u8> {
        let offset_str = offsets
            .iter()
            .map(|o| {
                if *o >= 0 {
                    format!("0x{:X}", o)
                } else {
                    format!("-0x{:X}", o.unsigned_abs())
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        format!("pointerPeek {} {}\r\n", offset_str, size).into_bytes()
    }

    /// Follow a pointer chain and write memory
    pub fn pointer_poke(offsets: &[i64], data: &[u8]) -> Vec<u8> {
        let offset_str = offsets
            .iter()
            .map(|o| {
                if *o >= 0 {
                    format!("0x{:X}", o)
                } else {
                    format!("-0x{:X}", o.unsigned_abs())
                }
            })
            .collect::<Vec<_>>()
            .join(" ");
        let hex_data = data.iter().map(|b| format!("{:02X}", b)).collect::<String>();
        format!("pointerPoke {} 0x{}\r\n", offset_str, hex_data).into_bytes()
    }

    /// Get the main NSO base address
    pub fn get_main_nso_base() -> Vec<u8> {
        b"getMainNsoBase\r\n".to_vec()
    }

    /// Get the heap base address
    pub fn get_heap_base() -> Vec<u8> {
        b"getHeapBase\r\n".to_vec()
    }

    /// Get the current game's title ID
    pub fn get_title_id() -> Vec<u8> {
        b"getTitleID\r\n".to_vec()
    }

    /// Get sys-botbase version
    pub fn get_version() -> Vec<u8> {
        b"getVersion\r\n".to_vec()
    }

    /// Detach the virtual controller
    pub fn detach_controller() -> Vec<u8> {
        b"detachController\r\n".to_vec()
    }

    /// Check if a program is running
    pub fn is_program_running(program_id: u64) -> Vec<u8> {
        format!("isProgramRunning 0x{:016X}\r\n", program_id).into_bytes()
    }
}

/// Decoder for sys-botbase responses
pub struct ResponseDecoder;

impl ResponseDecoder {
    /// Decode a hex string response to bytes
    /// sys-botbase returns data as hex-encoded ASCII (e.g., "0A0B0C" -> [10, 11, 12])
    pub fn decode_hex(response: &[u8]) -> crate::Result<Vec<u8>> {
        // Strip any trailing newlines
        let response = Self::strip_newlines(response);

        if response.is_empty() {
            return Ok(Vec::new());
        }

        // Check for even length (each byte = 2 hex chars)
        if response.len() % 2 != 0 {
            return Err(crate::Error::HexDecodeError(format!(
                "Invalid hex length: {} (must be even)",
                response.len()
            )));
        }

        let mut result = Vec::with_capacity(response.len() / 2);

        for chunk in response.chunks(2) {
            let high = Self::hex_char_to_nibble(chunk[0])?;
            let low = Self::hex_char_to_nibble(chunk[1])?;
            result.push((high << 4) | low);
        }

        Ok(result)
    }

    /// Decode a hex string response to a u64 (for addresses)
    pub fn decode_hex_u64(response: &[u8]) -> crate::Result<u64> {
        let response = Self::strip_newlines(response);

        let hex_str = std::str::from_utf8(response)
            .map_err(|e| crate::Error::HexDecodeError(e.to_string()))?;

        // Handle "0x" prefix if present
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let hex_str = hex_str.strip_prefix("0X").unwrap_or(hex_str);

        u64::from_str_radix(hex_str, 16)
            .map_err(|e| crate::Error::HexDecodeError(e.to_string()))
    }

    /// Strip trailing CRLF or LF
    fn strip_newlines(data: &[u8]) -> &[u8] {
        let mut end = data.len();
        while end > 0 && (data[end - 1] == b'\n' || data[end - 1] == b'\r') {
            end -= 1;
        }
        &data[..end]
    }

    /// Convert a single hex character to its nibble value
    fn hex_char_to_nibble(c: u8) -> crate::Result<u8> {
        match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'A'..=b'F' => Ok(c - b'A' + 10),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            _ => Err(crate::Error::HexDecodeError(format!(
                "Invalid hex character: '{}'",
                c as char
            ))),
        }
    }
}

// ============================================================================
// TESTS - TDD: These tests were written BEFORE the implementation
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Button Command Tests
    // ========================================================================

    #[test]
    fn test_click_button_a() {
        let cmd = SwitchCommand::click(SwitchButton::A);
        assert_eq!(cmd, b"click A\r\n");
    }

    #[test]
    fn test_click_button_b() {
        let cmd = SwitchCommand::click(SwitchButton::B);
        assert_eq!(cmd, b"click B\r\n");
    }

    #[test]
    fn test_click_button_x() {
        let cmd = SwitchCommand::click(SwitchButton::X);
        assert_eq!(cmd, b"click X\r\n");
    }

    #[test]
    fn test_click_button_y() {
        let cmd = SwitchCommand::click(SwitchButton::Y);
        assert_eq!(cmd, b"click Y\r\n");
    }

    #[test]
    fn test_click_dpad() {
        assert_eq!(SwitchCommand::click(SwitchButton::DUp), b"click DUP\r\n");
        assert_eq!(SwitchCommand::click(SwitchButton::DDown), b"click DDOWN\r\n");
        assert_eq!(SwitchCommand::click(SwitchButton::DLeft), b"click DLEFT\r\n");
        assert_eq!(SwitchCommand::click(SwitchButton::DRight), b"click DRIGHT\r\n");
    }

    #[test]
    fn test_click_triggers() {
        assert_eq!(SwitchCommand::click(SwitchButton::L), b"click L\r\n");
        assert_eq!(SwitchCommand::click(SwitchButton::R), b"click R\r\n");
        assert_eq!(SwitchCommand::click(SwitchButton::ZL), b"click ZL\r\n");
        assert_eq!(SwitchCommand::click(SwitchButton::ZR), b"click ZR\r\n");
    }

    #[test]
    fn test_press_and_release() {
        let press = SwitchCommand::press(SwitchButton::A);
        let release = SwitchCommand::release(SwitchButton::A);

        assert_eq!(press, b"press A\r\n");
        assert_eq!(release, b"release A\r\n");
    }

    // ========================================================================
    // Stick Command Tests
    // ========================================================================

    #[test]
    fn test_set_stick_left_center() {
        let cmd = SwitchCommand::set_stick(SwitchStick::Left, 0, 0);
        assert_eq!(cmd, b"setStick LEFT 0 0\r\n");
    }

    #[test]
    fn test_set_stick_right_max() {
        let cmd = SwitchCommand::set_stick(SwitchStick::Right, 32767, 32767);
        assert_eq!(cmd, b"setStick RIGHT 32767 32767\r\n");
    }

    #[test]
    fn test_set_stick_negative() {
        let cmd = SwitchCommand::set_stick(SwitchStick::Left, -32768, -32768);
        assert_eq!(cmd, b"setStick LEFT -32768 -32768\r\n");
    }

    #[test]
    fn test_reset_stick() {
        let cmd = SwitchCommand::reset_stick(SwitchStick::Left);
        assert_eq!(cmd, b"setStick LEFT 0 0\r\n");
    }

    // ========================================================================
    // Memory Command Tests
    // ========================================================================

    #[test]
    fn test_peek_heap() {
        let cmd = SwitchCommand::peek(0x12345678, 100, OffsetType::Heap);
        assert_eq!(cmd, b"peek 0x12345678 100\r\n");
    }

    #[test]
    fn test_peek_main() {
        let cmd = SwitchCommand::peek(0xABCDEF, 50, OffsetType::Main);
        assert_eq!(cmd, b"peekMain 0xABCDEF 50\r\n");
    }

    #[test]
    fn test_peek_absolute() {
        let cmd = SwitchCommand::peek(0xDEADBEEF, 8, OffsetType::Absolute);
        assert_eq!(cmd, b"peekAbsolute 0xDEADBEEF 8\r\n");
    }

    #[test]
    fn test_poke_heap() {
        let cmd = SwitchCommand::poke(0x1000, &[0x01, 0x02, 0x03], OffsetType::Heap);
        assert_eq!(cmd, b"poke 0x1000 0x010203\r\n");
    }

    #[test]
    fn test_poke_main() {
        let cmd = SwitchCommand::poke(0x2000, &[0xFF, 0x00, 0xAB], OffsetType::Main);
        assert_eq!(cmd, b"pokeMain 0x2000 0xFF00AB\r\n");
    }

    #[test]
    fn test_poke_empty_data() {
        let cmd = SwitchCommand::poke(0x1000, &[], OffsetType::Heap);
        assert_eq!(cmd, b"poke 0x1000 0x\r\n");
    }

    // ========================================================================
    // Pointer Command Tests
    // ========================================================================

    #[test]
    fn test_pointer_peek_single() {
        let cmd = SwitchCommand::pointer_peek(&[0x100], 8);
        assert_eq!(cmd, b"pointerPeek 0x100 8\r\n");
    }

    #[test]
    fn test_pointer_peek_chain() {
        let cmd = SwitchCommand::pointer_peek(&[0x100, 0x20, 0x8], 4);
        assert_eq!(cmd, b"pointerPeek 0x100 0x20 0x8 4\r\n");
    }

    #[test]
    fn test_pointer_peek_negative_offset() {
        let cmd = SwitchCommand::pointer_peek(&[0x100, -0x10], 8);
        assert_eq!(cmd, b"pointerPeek 0x100 -0x10 8\r\n");
    }

    #[test]
    fn test_pointer_poke() {
        let cmd = SwitchCommand::pointer_poke(&[0x100, 0x20], &[0xAB, 0xCD]);
        assert_eq!(cmd, b"pointerPoke 0x100 0x20 0xABCD\r\n");
    }

    // ========================================================================
    // System Command Tests
    // ========================================================================

    #[test]
    fn test_get_main_nso_base() {
        let cmd = SwitchCommand::get_main_nso_base();
        assert_eq!(cmd, b"getMainNsoBase\r\n");
    }

    #[test]
    fn test_get_heap_base() {
        let cmd = SwitchCommand::get_heap_base();
        assert_eq!(cmd, b"getHeapBase\r\n");
    }

    #[test]
    fn test_get_title_id() {
        let cmd = SwitchCommand::get_title_id();
        assert_eq!(cmd, b"getTitleID\r\n");
    }

    #[test]
    fn test_get_version() {
        let cmd = SwitchCommand::get_version();
        assert_eq!(cmd, b"getVersion\r\n");
    }

    #[test]
    fn test_detach_controller() {
        let cmd = SwitchCommand::detach_controller();
        assert_eq!(cmd, b"detachController\r\n");
    }

    #[test]
    fn test_is_program_running() {
        // Legends Z-A title ID (placeholder - actual ID TBD)
        let cmd = SwitchCommand::is_program_running(0x0100ABF008968000);
        assert_eq!(cmd, b"isProgramRunning 0x0100ABF008968000\r\n");
    }

    // ========================================================================
    // Response Decoder Tests
    // ========================================================================

    #[test]
    fn test_decode_hex_simple() {
        let result = ResponseDecoder::decode_hex(b"010203").unwrap();
        assert_eq!(result, vec![0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_decode_hex_uppercase() {
        let result = ResponseDecoder::decode_hex(b"AABBCC").unwrap();
        assert_eq!(result, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_decode_hex_lowercase() {
        let result = ResponseDecoder::decode_hex(b"aabbcc").unwrap();
        assert_eq!(result, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_decode_hex_mixed_case() {
        let result = ResponseDecoder::decode_hex(b"AaBbCc").unwrap();
        assert_eq!(result, vec![0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_decode_hex_with_crlf() {
        let result = ResponseDecoder::decode_hex(b"0A0B0C\r\n").unwrap();
        assert_eq!(result, vec![0x0A, 0x0B, 0x0C]);
    }

    #[test]
    fn test_decode_hex_with_lf() {
        let result = ResponseDecoder::decode_hex(b"0A0B0C\n").unwrap();
        assert_eq!(result, vec![0x0A, 0x0B, 0x0C]);
    }

    #[test]
    fn test_decode_hex_empty() {
        let result = ResponseDecoder::decode_hex(b"").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_decode_hex_only_newlines() {
        let result = ResponseDecoder::decode_hex(b"\r\n").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_decode_hex_odd_length_fails() {
        let result = ResponseDecoder::decode_hex(b"ABC");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_hex_invalid_char_fails() {
        let result = ResponseDecoder::decode_hex(b"GHIJ");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_hex_u64() {
        let result = ResponseDecoder::decode_hex_u64(b"DEADBEEF").unwrap();
        assert_eq!(result, 0xDEADBEEF);
    }

    #[test]
    fn test_decode_hex_u64_with_prefix() {
        let result = ResponseDecoder::decode_hex_u64(b"0xDEADBEEF").unwrap();
        assert_eq!(result, 0xDEADBEEF);
    }

    #[test]
    fn test_decode_hex_u64_full() {
        let result = ResponseDecoder::decode_hex_u64(b"0x0100ABF008968000\r\n").unwrap();
        assert_eq!(result, 0x0100ABF008968000);
    }

    // ========================================================================
    // Button Display Tests
    // ========================================================================

    #[test]
    fn test_button_display() {
        assert_eq!(format!("{}", SwitchButton::A), "A");
        assert_eq!(format!("{}", SwitchButton::Plus), "PLUS");
        assert_eq!(format!("{}", SwitchButton::DUp), "DUP");
        assert_eq!(format!("{}", SwitchButton::LStick), "LSTICK");
    }

    #[test]
    fn test_stick_display() {
        assert_eq!(format!("{}", SwitchStick::Left), "LEFT");
        assert_eq!(format!("{}", SwitchStick::Right), "RIGHT");
    }
}
