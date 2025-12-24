//! Connection traits defining the interface for Switch communication

use async_trait::async_trait;

use crate::command::{OffsetType, SwitchButton, SwitchStick};
use crate::Result;

/// Core trait for Switch connections
///
/// This trait defines all operations that can be performed on a connected
/// Nintendo Switch running sys-botbase or usb-botbase.
#[async_trait]
pub trait SwitchConnection: Send + Sync {
    // ========================================================================
    // Connection Management
    // ========================================================================

    /// Connect to the Switch
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the Switch
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if currently connected
    fn is_connected(&self) -> bool;

    /// Get the connection name/label
    fn name(&self) -> &str;

    /// Reset the connection (disconnect and reconnect)
    async fn reset(&mut self) -> Result<()> {
        self.disconnect().await?;
        self.connect().await
    }

    // ========================================================================
    // Raw Communication
    // ========================================================================

    /// Send raw bytes to the Switch
    async fn send(&mut self, data: &[u8]) -> Result<()>;

    /// Receive raw bytes from the Switch
    async fn receive(&mut self, size: usize) -> Result<Vec<u8>>;

    /// Send a command and receive the response
    async fn send_command(&mut self, command: &[u8]) -> Result<Vec<u8>>;

    // ========================================================================
    // Button Controls
    // ========================================================================

    /// Click a button (press and release)
    async fn click(&mut self, button: SwitchButton) -> Result<()> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::click(button);
        self.send(&cmd).await
    }

    /// Press and hold a button
    async fn press(&mut self, button: SwitchButton) -> Result<()> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::press(button);
        self.send(&cmd).await
    }

    /// Release a held button
    async fn release(&mut self, button: SwitchButton) -> Result<()> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::release(button);
        self.send(&cmd).await
    }

    /// Click a button with a delay after
    async fn click_with_delay(&mut self, button: SwitchButton, delay_ms: u64) -> Result<()> {
        self.click(button).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        Ok(())
    }

    /// Press and hold a button for a duration, then release
    async fn hold(&mut self, button: SwitchButton, duration_ms: u64) -> Result<()> {
        self.press(button).await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
        self.release(button).await
    }

    // ========================================================================
    // Stick Controls
    // ========================================================================

    /// Set analog stick position
    async fn set_stick(&mut self, stick: SwitchStick, x: i16, y: i16) -> Result<()> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::set_stick(stick, x, y);
        self.send(&cmd).await
    }

    /// Reset stick to center
    async fn reset_stick(&mut self, stick: SwitchStick) -> Result<()> {
        self.set_stick(stick, 0, 0).await
    }

    // ========================================================================
    // Memory Operations
    // ========================================================================

    /// Read bytes from memory
    async fn read_bytes(
        &mut self,
        offset: u64,
        size: usize,
        offset_type: OffsetType,
    ) -> Result<Vec<u8>> {
        use crate::command::{ResponseDecoder, SwitchCommand};
        let cmd = SwitchCommand::peek(offset, size, offset_type);
        let response = self.send_command(&cmd).await?;
        ResponseDecoder::decode_hex(&response)
    }

    /// Write bytes to memory
    async fn write_bytes(
        &mut self,
        offset: u64,
        data: &[u8],
        offset_type: OffsetType,
    ) -> Result<()> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::poke(offset, data, offset_type);
        self.send(&cmd).await
    }

    /// Read bytes following a pointer chain
    async fn read_pointer(&mut self, offsets: &[i64], size: usize) -> Result<Vec<u8>> {
        use crate::command::{ResponseDecoder, SwitchCommand};
        let cmd = SwitchCommand::pointer_peek(offsets, size);
        let response = self.send_command(&cmd).await?;
        ResponseDecoder::decode_hex(&response)
    }

    /// Write bytes following a pointer chain
    async fn write_pointer(&mut self, offsets: &[i64], data: &[u8]) -> Result<()> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::pointer_poke(offsets, data);
        self.send(&cmd).await
    }

    // ========================================================================
    // System Information
    // ========================================================================

    /// Get the main NSO base address
    async fn get_main_nso_base(&mut self) -> Result<u64> {
        use crate::command::{ResponseDecoder, SwitchCommand};
        let cmd = SwitchCommand::get_main_nso_base();
        let response = self.send_command(&cmd).await?;
        ResponseDecoder::decode_hex_u64(&response)
    }

    /// Get the heap base address
    async fn get_heap_base(&mut self) -> Result<u64> {
        use crate::command::{ResponseDecoder, SwitchCommand};
        let cmd = SwitchCommand::get_heap_base();
        let response = self.send_command(&cmd).await?;
        ResponseDecoder::decode_hex_u64(&response)
    }

    /// Get the current game's title ID
    async fn get_title_id(&mut self) -> Result<u64> {
        use crate::command::{ResponseDecoder, SwitchCommand};
        let cmd = SwitchCommand::get_title_id();
        let response = self.send_command(&cmd).await?;
        ResponseDecoder::decode_hex_u64(&response)
    }

    /// Get sys-botbase version
    async fn get_version(&mut self) -> Result<String> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::get_version();
        let response = self.send_command(&cmd).await?;
        // Version is returned as ASCII text
        String::from_utf8(response)
            .map(|s| s.trim().to_string())
            .map_err(|e| crate::Error::InvalidResponse {
                message: e.to_string(),
            })
    }

    /// Check if a specific program is running
    async fn is_program_running(&mut self, program_id: u64) -> Result<bool> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::is_program_running(program_id);
        let response = self.send_command(&cmd).await?;
        // Response is "1" or "0"
        Ok(response.first() == Some(&b'1'))
    }

    /// Detach the virtual controller
    async fn detach_controller(&mut self) -> Result<()> {
        use crate::command::SwitchCommand;
        let cmd = SwitchCommand::detach_controller();
        self.send(&cmd).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    /// Mock connection for testing the trait default implementations
    pub struct TestConnection {
        connected: bool,
        name: String,
        sent_commands: Arc<Mutex<Vec<Vec<u8>>>>,
        responses: Arc<Mutex<VecDeque<Vec<u8>>>>,
    }

    impl TestConnection {
        pub fn new() -> Self {
            Self {
                connected: false,
                name: "TestSwitch".to_string(),
                sent_commands: Arc::new(Mutex::new(Vec::new())),
                responses: Arc::new(Mutex::new(VecDeque::new())),
            }
        }

        pub fn with_response(self, response: &[u8]) -> Self {
            self.responses.lock().unwrap().push_back(response.to_vec());
            self
        }

        pub fn get_sent_commands(&self) -> Vec<Vec<u8>> {
            self.sent_commands.lock().unwrap().clone()
        }
    }

    #[async_trait]
    impl SwitchConnection for TestConnection {
        async fn connect(&mut self) -> Result<()> {
            self.connected = true;
            Ok(())
        }

        async fn disconnect(&mut self) -> Result<()> {
            self.connected = false;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.connected
        }

        fn name(&self) -> &str {
            &self.name
        }

        async fn send(&mut self, data: &[u8]) -> Result<()> {
            if !self.connected {
                return Err(crate::Error::NotConnected);
            }
            self.sent_commands.lock().unwrap().push(data.to_vec());
            Ok(())
        }

        async fn receive(&mut self, _size: usize) -> Result<Vec<u8>> {
            if !self.connected {
                return Err(crate::Error::NotConnected);
            }
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or(crate::Error::Timeout { timeout_ms: 5000 })
        }

        async fn send_command(&mut self, command: &[u8]) -> Result<Vec<u8>> {
            self.send(command).await?;
            self.receive(0).await
        }
    }

    #[tokio::test]
    async fn test_connect_disconnect() {
        let mut conn = TestConnection::new();

        assert!(!conn.is_connected());
        conn.connect().await.unwrap();
        assert!(conn.is_connected());
        conn.disconnect().await.unwrap();
        assert!(!conn.is_connected());
    }

    #[tokio::test]
    async fn test_reset_connection() {
        let mut conn = TestConnection::new();
        conn.connect().await.unwrap();

        conn.reset().await.unwrap();
        assert!(conn.is_connected());
    }

    #[tokio::test]
    async fn test_click_button() {
        let mut conn = TestConnection::new();
        conn.connect().await.unwrap();

        conn.click(SwitchButton::A).await.unwrap();

        let sent = conn.get_sent_commands();
        assert_eq!(sent.len(), 1);
        assert_eq!(sent[0], b"click A\r\n");
    }

    #[tokio::test]
    async fn test_press_and_release() {
        let mut conn = TestConnection::new();
        conn.connect().await.unwrap();

        conn.press(SwitchButton::B).await.unwrap();
        conn.release(SwitchButton::B).await.unwrap();

        let sent = conn.get_sent_commands();
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0], b"press B\r\n");
        assert_eq!(sent[1], b"release B\r\n");
    }

    #[tokio::test]
    async fn test_set_stick() {
        let mut conn = TestConnection::new();
        conn.connect().await.unwrap();

        conn.set_stick(SwitchStick::Left, 100, -200).await.unwrap();

        let sent = conn.get_sent_commands();
        assert_eq!(sent[0], b"setStick LEFT 100 -200\r\n");
    }

    #[tokio::test]
    async fn test_read_bytes() {
        let mut conn = TestConnection::new()
            .with_response(b"AABBCCDD\r\n");
        conn.connect().await.unwrap();

        let data = conn
            .read_bytes(0x1000, 4, OffsetType::Heap)
            .await
            .unwrap();

        assert_eq!(data, vec![0xAA, 0xBB, 0xCC, 0xDD]);

        let sent = conn.get_sent_commands();
        assert_eq!(sent[0], b"peek 0x1000 4\r\n");
    }

    #[tokio::test]
    async fn test_write_bytes() {
        let mut conn = TestConnection::new();
        conn.connect().await.unwrap();

        conn.write_bytes(0x2000, &[0x11, 0x22], OffsetType::Main)
            .await
            .unwrap();

        let sent = conn.get_sent_commands();
        assert_eq!(sent[0], b"pokeMain 0x2000 0x1122\r\n");
    }

    #[tokio::test]
    async fn test_read_pointer() {
        let mut conn = TestConnection::new()
            .with_response(b"12345678\r\n");
        conn.connect().await.unwrap();

        let data = conn.read_pointer(&[0x100, 0x20, 0x8], 4).await.unwrap();

        assert_eq!(data, vec![0x12, 0x34, 0x56, 0x78]);

        let sent = conn.get_sent_commands();
        assert_eq!(sent[0], b"pointerPeek 0x100 0x20 0x8 4\r\n");
    }

    #[tokio::test]
    async fn test_get_title_id() {
        let mut conn = TestConnection::new()
            .with_response(b"0100ABF008968000\r\n");
        conn.connect().await.unwrap();

        let title_id = conn.get_title_id().await.unwrap();

        assert_eq!(title_id, 0x0100ABF008968000);
    }

    #[tokio::test]
    async fn test_get_version() {
        let mut conn = TestConnection::new()
            .with_response(b"2.4.0\r\n");
        conn.connect().await.unwrap();

        let version = conn.get_version().await.unwrap();

        assert_eq!(version, "2.4.0");
    }

    #[tokio::test]
    async fn test_is_program_running() {
        let mut conn = TestConnection::new()
            .with_response(b"1\r\n");
        conn.connect().await.unwrap();

        let running = conn.is_program_running(0x0100ABF008968000).await.unwrap();
        assert!(running);
    }

    #[tokio::test]
    async fn test_not_connected_error() {
        let mut conn = TestConnection::new();
        // Don't connect

        let result = conn.click(SwitchButton::A).await;
        assert!(matches!(result, Err(crate::Error::NotConnected)));
    }

    #[tokio::test]
    async fn test_detach_controller() {
        let mut conn = TestConnection::new();
        conn.connect().await.unwrap();

        conn.detach_controller().await.unwrap();

        let sent = conn.get_sent_commands();
        assert_eq!(sent[0], b"detachController\r\n");
    }
}
