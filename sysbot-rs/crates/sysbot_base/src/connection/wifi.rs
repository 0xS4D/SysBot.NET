//! WiFi connection implementation using TCP sockets

use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::{timeout, Duration};

use super::{ConnectionConfig, SwitchConnection};
use crate::command::OffsetType;
use crate::{Error, Result};

/// WiFi connection to a Nintendo Switch running sys-botbase
pub struct WifiConnection {
    config: ConnectionConfig,
    stream: Option<Arc<Mutex<TcpStream>>>,
}

impl WifiConnection {
    /// Create a new WiFi connection with the given configuration
    pub fn new(config: ConnectionConfig) -> Self {
        Self {
            config,
            stream: None,
        }
    }

    /// Create a WiFi connection from IP and port
    pub fn from_ip(ip: &str, port: u16) -> Result<Self> {
        let config = ConnectionConfig::wifi(ip, port)?;
        Ok(Self::new(config))
    }

    /// Get the connection configuration
    pub fn config(&self) -> &ConnectionConfig {
        &self.config
    }

    /// Internal helper to get the stream with connection check
    fn stream(&self) -> Result<Arc<Mutex<TcpStream>>> {
        self.stream.clone().ok_or(Error::NotConnected)
    }

    /// Read response from the socket until we get a complete line
    async fn read_response(&self) -> Result<Vec<u8>> {
        let stream = self.stream()?;
        let mut stream = stream.lock().await;
        let mut buffer = Vec::with_capacity(1024);
        let mut byte = [0u8; 1];

        let timeout_duration = Duration::from_millis(self.config.timeout_ms);

        loop {
            match timeout(timeout_duration, stream.read_exact(&mut byte)).await {
                Ok(Ok(_)) => {
                    buffer.push(byte[0]);
                    // Check for CRLF or just LF
                    if byte[0] == b'\n' {
                        break;
                    }
                }
                Ok(Err(e)) => {
                    return Err(Error::ConnectionLost(e.to_string()));
                }
                Err(_) => {
                    return Err(Error::Timeout {
                        timeout_ms: self.config.timeout_ms,
                    });
                }
            }
        }

        Ok(buffer)
    }
}

#[async_trait]
impl SwitchConnection for WifiConnection {
    async fn connect(&mut self) -> Result<()> {
        if self.stream.is_some() {
            return Ok(()); // Already connected
        }

        tracing::info!(
            address = %self.config.address,
            name = %self.config.name,
            "Connecting to Switch via WiFi"
        );

        let timeout_duration = Duration::from_millis(self.config.timeout_ms);

        let stream = match timeout(timeout_duration, TcpStream::connect(self.config.address)).await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                return Err(Error::ConnectionFailed {
                    address: self.config.address.to_string(),
                    source: e,
                });
            }
            Err(_) => {
                return Err(Error::Timeout {
                    timeout_ms: self.config.timeout_ms,
                });
            }
        };

        // Disable Nagle's algorithm for lower latency
        stream.set_nodelay(true)?;

        self.stream = Some(Arc::new(Mutex::new(stream)));

        tracing::info!(
            name = %self.config.name,
            "Successfully connected to Switch"
        );

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        if let Some(stream) = self.stream.take() {
            let mut stream = stream.lock().await;
            // Shutdown is best-effort
            let _ = stream.shutdown().await;
            tracing::info!(name = %self.config.name, "Disconnected from Switch");
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    fn name(&self) -> &str {
        &self.config.name
    }

    async fn send(&mut self, data: &[u8]) -> Result<()> {
        let stream = self.stream()?;
        let mut stream = stream.lock().await;

        tracing::trace!(
            command = %String::from_utf8_lossy(data).trim(),
            "Sending command"
        );

        stream.write_all(data).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to send command");
            Error::ConnectionLost(e.to_string())
        })?;

        // Add base delay after sending
        if self.config.base_delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.config.base_delay_ms)).await;
        }

        Ok(())
    }

    async fn receive(&mut self, _size: usize) -> Result<Vec<u8>> {
        self.read_response().await
    }

    async fn send_command(&mut self, command: &[u8]) -> Result<Vec<u8>> {
        self.send(command).await?;
        self.read_response().await
    }

    // Override read_bytes for chunked large transfers
    async fn read_bytes(
        &mut self,
        offset: u64,
        size: usize,
        offset_type: OffsetType,
    ) -> Result<Vec<u8>> {
        use crate::command::{ResponseDecoder, SwitchCommand};

        let max_chunk = self.config.max_transfer_size;

        if size <= max_chunk {
            // Small read - single command
            let cmd = SwitchCommand::peek(offset, size, offset_type);
            let response = self.send_command(&cmd).await?;
            return ResponseDecoder::decode_hex(&response);
        }

        // Large read - chunk it
        let mut result = Vec::with_capacity(size);
        let mut remaining = size;
        let mut current_offset = offset;

        while remaining > 0 {
            let chunk_size = remaining.min(max_chunk);
            let cmd = SwitchCommand::peek(current_offset, chunk_size, offset_type);
            let response = self.send_command(&cmd).await?;
            let decoded = ResponseDecoder::decode_hex(&response)?;

            result.extend_from_slice(&decoded);
            remaining -= chunk_size;
            current_offset += chunk_size as u64;

            // Small delay between chunks
            if remaining > 0 {
                tokio::time::sleep(Duration::from_millis(self.config.base_delay_ms / 2)).await;
            }
        }

        Ok(result)
    }

    // Override write_bytes for chunked large transfers
    async fn write_bytes(
        &mut self,
        offset: u64,
        data: &[u8],
        offset_type: OffsetType,
    ) -> Result<()> {
        use crate::command::SwitchCommand;

        let max_chunk = self.config.max_transfer_size;

        if data.len() <= max_chunk {
            // Small write - single command
            let cmd = SwitchCommand::poke(offset, data, offset_type);
            return self.send(&cmd).await;
        }

        // Large write - chunk it
        let mut current_offset = offset;

        for chunk in data.chunks(max_chunk) {
            let cmd = SwitchCommand::poke(current_offset, chunk, offset_type);
            self.send(&cmd).await?;
            current_offset += chunk.len() as u64;

            // Small delay between chunks
            tokio::time::sleep(Duration::from_millis(self.config.base_delay_ms / 2)).await;
        }

        Ok(())
    }
}

impl std::fmt::Debug for WifiConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WifiConnection")
            .field("config", &self.config)
            .field("connected", &self.is_connected())
            .finish()
    }
}

// ============================================================================
// Mock Connection for Testing
// ============================================================================

/// Mock connection for testing without a real Switch
#[cfg(test)]
pub struct MockConnection {
    connected: bool,
    name: String,
    commands: std::sync::Arc<std::sync::Mutex<Vec<Vec<u8>>>>,
    responses: std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<Vec<u8>>>>,
}

#[cfg(test)]
impl MockConnection {
    pub fn new() -> Self {
        Self {
            connected: false,
            name: "MockSwitch".to_string(),
            commands: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            responses: std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new())),
        }
    }

    pub fn queue_response(&mut self, response: &[u8]) {
        self.responses.lock().unwrap().push_back(response.to_vec());
    }

    pub fn get_commands(&self) -> Vec<Vec<u8>> {
        self.commands.lock().unwrap().clone()
    }
}

#[cfg(test)]
#[async_trait]
impl SwitchConnection for MockConnection {
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
            return Err(Error::NotConnected);
        }
        self.commands.lock().unwrap().push(data.to_vec());
        Ok(())
    }

    async fn receive(&mut self, _size: usize) -> Result<Vec<u8>> {
        if !self.connected {
            return Err(Error::NotConnected);
        }
        self.responses
            .lock()
            .unwrap()
            .pop_front()
            .ok_or(Error::Timeout { timeout_ms: 5000 })
    }

    async fn send_command(&mut self, command: &[u8]) -> Result<Vec<u8>> {
        self.send(command).await?;
        self.receive(0).await
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::SwitchButton;

    #[test]
    fn test_wifi_connection_creation() {
        let conn = WifiConnection::from_ip("192.168.1.100", 6000).unwrap();
        assert!(!conn.is_connected());
        assert!(conn.name().contains("192.168.1.100"));
    }

    #[test]
    fn test_wifi_connection_config() {
        let config = ConnectionConfig::wifi("10.0.0.1", 6000)
            .unwrap()
            .with_name("TestSwitch");
        let conn = WifiConnection::new(config);

        assert_eq!(conn.name(), "TestSwitch");
        assert_eq!(conn.config().address.port(), 6000);
    }

    #[tokio::test]
    async fn test_mock_connection() {
        let mut conn = MockConnection::new();
        conn.queue_response(b"DEADBEEF\r\n");

        conn.connect().await.unwrap();
        assert!(conn.is_connected());

        let response = conn.send_command(b"test\r\n").await.unwrap();
        assert_eq!(response, b"DEADBEEF\r\n");

        let commands = conn.get_commands();
        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0], b"test\r\n");
    }

    #[tokio::test]
    async fn test_mock_button_sequence() {
        let mut conn = MockConnection::new();
        conn.connect().await.unwrap();

        // Simulate a button combo: A, then B
        conn.click(SwitchButton::A).await.unwrap();
        conn.click(SwitchButton::B).await.unwrap();

        let commands = conn.get_commands();
        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0], b"click A\r\n");
        assert_eq!(commands[1], b"click B\r\n");
    }

    #[tokio::test]
    async fn test_mock_memory_read() {
        let mut conn = MockConnection::new();
        conn.queue_response(b"01020304\r\n");
        conn.connect().await.unwrap();

        let data = conn.read_bytes(0x1000, 4, OffsetType::Heap).await.unwrap();
        assert_eq!(data, vec![0x01, 0x02, 0x03, 0x04]);
    }

    #[tokio::test]
    async fn test_not_connected_error() {
        let mut conn = MockConnection::new();
        // Don't connect

        let result = conn.click(SwitchButton::A).await;
        assert!(matches!(result, Err(Error::NotConnected)));
    }
}
