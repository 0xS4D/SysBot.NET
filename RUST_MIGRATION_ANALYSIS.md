# SysBot.NET → Rust Migration Analysis

## Executive Summary

**Project:** SysBot.NET - Automated Pokemon Trading Bot Framework
**Current Stack:** C# / .NET 9.0
**Target Stack:** Rust
**Total C# Lines:** ~12,000+ lines across 219 files
**Estimated Rust Lines:** ~15,000-18,000 (Rust tends to be more verbose)

---

## 1. Project Overview

SysBot.NET is a comprehensive Pokemon automation framework that:
- Connects to Nintendo Switch consoles via sys-botbase (WiFi) or usb-botbase (USB)
- Automates Pokemon trades, raids, encounters, and egg hatching
- Integrates with Discord, Twitch, and YouTube for remote management
- Supports 5 Pokemon games: Sword/Shield, Scarlet/Violet, BDSP, Legends Arceus, Legends Z-A
- Uses PKHeX for Pokemon data validation and legality checking

---

## 2. Current Architecture (C#)

```
┌─────────────────────────────────────────────────────────────────┐
│                    Application Layer                             │
├─────────────────────┬───────────────────────────────────────────┤
│  WinForms GUI       │  Console App (Headless)                   │
│  - Bot Management   │  - Daemon Mode                            │
│  - Configuration    │  - Graceful Shutdown                      │
│  - Live Logging     │  - Signal Handling                        │
└─────────────────────┴───────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    Integration Layer                             │
├──────────────┬──────────────┬──────────────┬───────────────────┤
│   Discord    │    Twitch    │   YouTube    │       Z3          │
│   Bot        │    Bot       │   Bot        │   Seed Search     │
│   (Discord.  │   (TwitchLib)│   (Google    │   (PKHeX RNG)     │
│    Net)      │              │    APIs)     │                   │
└──────────────┴──────────────┴──────────────┴───────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    Pokemon Domain Layer                          │
├─────────────────────────────────────────────────────────────────┤
│  SysBot.Pokemon                                                  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │
│  │  TradeHub   │  │   Queues    │  │  Game-Specific Modules  │  │
│  │  - Central  │  │  - Priority │  │  - SWSH (PK8)           │  │
│  │    Coord.   │  │  - Favored  │  │  - SV (PK9)             │  │
│  │  - Ledy     │  │  - Multi-   │  │  - BDSP (PB8)           │  │
│  │    Distrib. │  │    type     │  │  - LA (PA8)             │  │
│  └─────────────┘  └─────────────┘  │  - LZA (PA9)            │  │
│                                    └─────────────────────────┘  │
│  ┌─────────────────────────────────────────────────────────────┐│
│  │  Routine Executors (PokeRoutineExecutor<T>)                 ││
│  │  - Trade Bots    - Encounter Bots   - Raid Bots             ││
│  │  - Egg Bots      - Fossil Bots      - Remote Control        ││
│  └─────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    Base Infrastructure Layer                     │
├─────────────────────────────────────────────────────────────────┤
│  SysBot.Base                                                     │
│  ┌────────────────────┐  ┌────────────────────────────────────┐ │
│  │  Connection Layer  │  │  Control Layer                     │ │
│  │  - SwitchSocket    │  │  - BotRunner                       │ │
│  │    (WiFi/TCP)      │  │  - BotSource                       │ │
│  │  - SwitchUSB       │  │  - RoutineExecutor                 │ │
│  │    (LibUsbDotNet)  │  │  - BotSynchronizer                 │ │
│  │  - Command Builder │  │  - State Management                │ │
│  └────────────────────┘  └────────────────────────────────────┘ │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  Utilities: Logging (NLog), Encoding, Echo, Records        │ │
│  └────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    External Dependencies                         │
├──────────────┬──────────────┬──────────────┬───────────────────┤
│   PKHeX.Core │  Discord.Net │  TwitchLib   │  LibUsbDotNet     │
│   (Pokemon   │  (Discord    │  (Twitch     │  (USB Device      │
│    Data)     │   API)       │   API)       │   Access)         │
└──────────────┴──────────────┴──────────────┴───────────────────┘
```

---

## 3. Module Mapping: C# → Rust

### 3.1 Core Modules

| C# Module | Rust Crate | Description |
|-----------|------------|-------------|
| `SysBot.Base` | `sysbot_base` | Connection, control, logging |
| `SysBot.Pokemon` | `sysbot_pokemon` | Pokemon domain logic |
| `SysBot.Pokemon.Discord` | `sysbot_discord` | Discord integration |
| `SysBot.Pokemon.Twitch` | `sysbot_twitch` | Twitch integration |
| `SysBot.Pokemon.YouTube` | `sysbot_youtube` | YouTube integration |
| `SysBot.Pokemon.Z3` | `sysbot_z3` | Seed searching |
| `SysBot.Pokemon.WinForms` | `sysbot_gui` (egui/iced) | GUI application |
| `SysBot.Pokemon.ConsoleApp` | `sysbot_cli` | CLI application |
| `SysBot.Tests` | `sysbot_tests` | Test suite |

### 3.2 Proposed Rust Workspace Structure

```
sysbot-rs/
├── Cargo.toml                    # Workspace definition
├── crates/
│   ├── sysbot_base/              # Core infrastructure
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── connection/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── switch_socket.rs      # WiFi connection
│   │   │   │   ├── switch_usb.rs         # USB connection
│   │   │   │   ├── switch_command.rs     # Command encoding
│   │   │   │   └── traits.rs             # Connection traits
│   │   │   ├── control/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── bot_runner.rs
│   │   │   │   ├── bot_state.rs
│   │   │   │   ├── routine_executor.rs
│   │   │   │   └── synchronizer.rs
│   │   │   ├── util/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── logging.rs
│   │   │   │   ├── decoder.rs
│   │   │   │   └── echo.rs
│   │   │   └── enums.rs                  # SwitchButton, etc.
│   │   └── Cargo.toml
│   │
│   ├── sysbot_pokemon/           # Pokemon domain
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── trade_hub/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── hub.rs
│   │   │   │   ├── queue.rs
│   │   │   │   ├── detail.rs
│   │   │   │   └── notifier.rs
│   │   │   ├── games/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── swsh/                 # Sword/Shield
│   │   │   │   ├── sv/                   # Scarlet/Violet
│   │   │   │   ├── bdsp/                 # BDSP
│   │   │   │   ├── la/                   # Legends Arceus
│   │   │   │   └── lza/                  # Legends Z-A
│   │   │   ├── bots/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── trade_bot.rs
│   │   │   │   ├── encounter_bot.rs
│   │   │   │   ├── raid_bot.rs
│   │   │   │   └── remote_control.rs
│   │   │   ├── settings/
│   │   │   │   └── mod.rs
│   │   │   └── helpers/
│   │   │       └── mod.rs
│   │   └── Cargo.toml
│   │
│   ├── sysbot_discord/           # Discord integration
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── bot.rs
│   │   │   ├── commands/
│   │   │   │   ├── mod.rs
│   │   │   │   ├── trade.rs
│   │   │   │   ├── queue.rs
│   │   │   │   └── admin.rs
│   │   │   └── helpers/
│   │   │       └── mod.rs
│   │   └── Cargo.toml
│   │
│   ├── sysbot_twitch/            # Twitch integration
│   ├── sysbot_youtube/           # YouTube integration
│   ├── sysbot_z3/                # Seed search
│   │
│   ├── sysbot_cli/               # CLI application
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   │
│   ├── sysbot_gui/               # GUI application (egui/iced)
│   │   ├── src/
│   │   │   └── main.rs
│   │   └── Cargo.toml
│   │
│   └── pkhex_rs/                 # PKHeX port (major effort)
│       ├── src/
│       │   ├── lib.rs
│       │   ├── pkm/              # Pokemon data structures
│       │   ├── legality/         # Legality checking
│       │   ├── saves/            # Save file handling
│       │   └── rng/              # RNG algorithms
│       └── Cargo.toml
│
├── tests/                        # Integration tests
└── examples/                     # Usage examples
```

---

## 4. Dependency Mapping: C# NuGet → Rust Crates

### 4.1 Direct Equivalents

| C# Package | Rust Crate | Notes |
|------------|------------|-------|
| `System.Net.Sockets` | `tokio::net::TcpStream` | Async TCP |
| `LibUsbDotNet` | `rusb` or `nusb` | USB device access |
| `Discord.Net` | `serenity` or `twilight` | Discord API |
| `TwitchLib.Client` | `twitch-irc` | Twitch chat |
| `Google.Apis.YouTube.v3` | `google-youtube3` | YouTube API |
| `NLog` | `tracing` + `tracing-subscriber` | Logging |
| `System.Text.Json` | `serde` + `serde_json` | JSON serialization |
| `xUnit` | Built-in `#[test]` + `rstest` | Testing |
| `FluentAssertions` | `assert_matches` or custom macros | Assertions |

### 4.2 Complex Dependencies Requiring Reimplementation

| C# Package | Strategy | Effort |
|------------|----------|--------|
| **PKHeX.Core** | Port to Rust (`pkhex_rs`) | **HIGH** - 50,000+ lines |
| **PKHeX.Core.AutoMod** | Port with `pkhex_rs` | **MEDIUM** |
| **Z3 (via PKHeX)** | Implement RNG algorithms | **MEDIUM** |

### 4.3 Recommended Rust Crates

```toml
# Cargo.toml (workspace)
[workspace.dependencies]

# Async Runtime
tokio = { version = "1.0", features = ["full"] }

# Networking
rusb = "0.9"                       # USB device access
nusb = "0.1"                       # Alternative USB (pure Rust)

# Discord
serenity = { version = "0.12", features = ["client", "gateway", "cache"] }
# OR
twilight-gateway = "0.15"
twilight-http = "0.15"

# Twitch
twitch-irc = "5.0"

# YouTube (Google APIs)
google-youtube3 = "5.0"
yup-oauth2 = "8.0"

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Logging
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }

# Error Handling
thiserror = "1.0"
anyhow = "1.0"

# Async Utilities
futures = "0.3"
async-trait = "0.1"

# GUI (choose one)
egui = "0.27"                      # Immediate mode GUI
eframe = "0.27"                    # egui framework
# OR
iced = "0.12"                      # Elm-inspired GUI

# CLI
clap = { version = "4.0", features = ["derive"] }

# Configuration
config = "0.14"

# Concurrency
parking_lot = "0.12"               # Fast mutexes
crossbeam = "0.8"                  # Concurrent data structures

# Testing
rstest = "0.18"                    # Parameterized tests
mockall = "0.12"                   # Mocking
```

---

## 5. Key Design Patterns to Migrate

### 5.1 Generic Bot System (C#)

```csharp
// C# - Generic over PKM type
public class PokeRoutineExecutor<T> where T : PKM, new()
{
    public async Task<T> ReadPokemon(ulong offset) { ... }
}
```

**Rust Equivalent:**

```rust
// Rust - Trait-based generics
pub trait Pokemon: Clone + Default + Send + Sync {
    fn species(&self) -> u16;
    fn from_bytes(data: &[u8]) -> Result<Self>;
    fn to_bytes(&self) -> Vec<u8>;
}

pub struct PokeRoutineExecutor<T: Pokemon> {
    connection: Box<dyn SwitchConnection>,
    _marker: PhantomData<T>,
}

impl<T: Pokemon> PokeRoutineExecutor<T> {
    pub async fn read_pokemon(&self, offset: u64) -> Result<T> { ... }
}
```

### 5.2 Connection Abstraction (C#)

```csharp
// C# - Async and Sync interfaces
public interface ISwitchConnectionAsync {
    Task<byte[]> ReadBytesAsync(uint offset, int length, CancellationToken ct);
    Task WriteBytesAsync(byte[] data, uint offset, CancellationToken ct);
}
```

**Rust Equivalent:**

```rust
// Rust - Async trait with async_trait macro
#[async_trait]
pub trait SwitchConnection: Send + Sync {
    async fn read_bytes(&self, offset: u32, length: usize) -> Result<Vec<u8>>;
    async fn write_bytes(&self, data: &[u8], offset: u32) -> Result<()>;
    async fn send_command(&self, cmd: &[u8]) -> Result<Vec<u8>>;

    fn is_connected(&self) -> bool;
    fn name(&self) -> &str;
}

pub struct WifiConnection {
    stream: TcpStream,
    config: ConnectionConfig,
}

#[async_trait]
impl SwitchConnection for WifiConnection {
    // Implementation
}
```

### 5.3 Trade Queue with Priorities (C#)

```csharp
// C# - Concurrent priority queue
public class FavoredCPQ<TPriority, TValue> { ... }
```

**Rust Equivalent:**

```rust
// Rust - Using crossbeam or custom implementation
use std::collections::BinaryHeap;
use parking_lot::Mutex;

pub struct PriorityQueue<T: Ord + Send> {
    heap: Mutex<BinaryHeap<T>>,
}

// Or use priority-queue crate
use priority_queue::PriorityQueue;
```

### 5.4 Notifier Pattern (C#)

```csharp
// C# - Interface-based notifications
public interface IPokeTradeNotifier<T> {
    void TradeInitialize(...);
    void TradeSearching(...);
    void TradeFinished(...);
}
```

**Rust Equivalent:**

```rust
// Rust - Trait with async methods
#[async_trait]
pub trait TradeNotifier<T: Pokemon>: Send + Sync {
    async fn trade_initialize(&self, detail: &TradeDetail<T>);
    async fn trade_searching(&self, detail: &TradeDetail<T>);
    async fn trade_finished(&self, detail: &TradeDetail<T>, result: &T);
    async fn trade_canceled(&self, detail: &TradeDetail<T>, reason: TradeResult);
}

// Platform-specific implementations
pub struct DiscordNotifier { /* ... */ }
pub struct TwitchNotifier { /* ... */ }
```

### 5.5 Factory Pattern (C#)

```csharp
// C# - Bot factory
public interface IBotFactory<T> {
    PokeRoutineExecutorBase CreateBot(PokeTradeHub<T> hub, PokeBotState cfg);
}
```

**Rust Equivalent:**

```rust
// Rust - Factory trait
pub trait BotFactory<T: Pokemon>: Send + Sync {
    fn create_bot(
        &self,
        hub: Arc<TradeHub<T>>,
        config: BotConfig,
    ) -> Result<Box<dyn RoutineExecutor<T>>>;

    fn supports_routine(&self, routine: RoutineType) -> bool;
}

// Game-specific factories
pub struct SwshBotFactory;
pub struct SvBotFactory;

impl BotFactory<PK8> for SwshBotFactory { /* ... */ }
impl BotFactory<PK9> for SvBotFactory { /* ... */ }
```

---

## 6. Technical Challenges

### 6.1 PKHeX Port (Critical Path)

**Challenge:** PKHeX.Core is ~50,000+ lines of C# with complex Pokemon data structures, legality checking, and encounter databases.

**Options:**
1. **Full Port** - Most work but cleanest solution
2. **FFI Bridge** - Call C# via interop (complex, defeats purpose)
3. **Minimal Port** - Only port what's needed for trading

**Recommended Approach:** Minimal port with expansion as needed

```rust
// Minimal PKM implementation
pub struct PK8 {
    data: [u8; 344],  // Raw Pokemon data
}

impl Pokemon for PK8 {
    fn species(&self) -> u16 {
        u16::from_le_bytes([self.data[8], self.data[9]])
    }

    fn encryption_constant(&self) -> u32 {
        u32::from_le_bytes(self.data[0..4].try_into().unwrap())
    }

    // Only implement needed methods
}
```

### 6.2 Async Runtime Consistency

**Challenge:** C# uses Task-based async; Rust has multiple runtimes.

**Solution:** Standardize on Tokio throughout

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // Single runtime for entire application
}
```

### 6.3 Cross-Platform GUI

**Challenge:** WinForms is Windows-only.

**Options:**
1. **egui/eframe** - Immediate mode, easy, cross-platform
2. **iced** - Elm-like, more complex but powerful
3. **tauri** - Web-based UI with Rust backend

**Recommendation:** egui for simplicity

```rust
// egui example
impl eframe::App for SysBotApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("SysBot.rs");

            if ui.button("Add Bot").clicked() {
                self.add_bot();
            }

            for bot in &self.bots {
                ui.horizontal(|ui| {
                    ui.label(&bot.name);
                    ui.label(format!("{:?}", bot.status));
                });
            }
        });
    }
}
```

### 6.4 USB Device Access

**Challenge:** LibUsbDotNet → Rust USB libraries

**Options:**
1. `rusb` - Wrapper around libusb (requires libusb installed)
2. `nusb` - Pure Rust, cross-platform, newer

**Recommendation:** `nusb` for pure Rust solution

```rust
use nusb::Device;

pub struct UsbConnection {
    device: Device,
    endpoint_in: u8,
    endpoint_out: u8,
}

impl UsbConnection {
    pub async fn connect() -> Result<Self> {
        let device = nusb::list_devices()?
            .find(|d| d.vendor_id() == 0x057E && d.product_id() == 0x3000)
            .ok_or(Error::DeviceNotFound)?
            .open()?;

        // Setup endpoints...
        Ok(Self { /* ... */ })
    }
}
```

### 6.5 Memory Safety with Raw Pointers

**Challenge:** C# uses unsafe blocks for memory operations; Rust is strict.

**Solution:** Encapsulate unsafe in safe abstractions

```rust
// Safe wrapper for pointer operations
pub struct PointerChain {
    base: u64,
    offsets: Vec<i64>,
}

impl PointerChain {
    pub async fn resolve(&self, conn: &impl SwitchConnection) -> Result<u64> {
        let mut addr = self.base;
        for &offset in &self.offsets {
            let bytes = conn.read_bytes_absolute(addr, 8).await?;
            addr = u64::from_le_bytes(bytes.try_into()?) as i64 + offset;
        }
        Ok(addr as u64)
    }
}
```

---

## 7. Migration Strategy

### Phase 1: Foundation (Weeks 1-4)
1. Set up Rust workspace structure
2. Implement `sysbot_base`:
   - Connection traits and WiFi implementation
   - Command encoding/decoding
   - Basic logging with tracing
3. Implement minimal `pkhex_rs`:
   - PK8, PK9, PB8, PA8, PA9 data structures
   - Basic legality checks

### Phase 2: Core Pokemon Logic (Weeks 5-8)
1. Implement `sysbot_pokemon`:
   - TradeHub and queue system
   - Notifier traits
   - Trade detail structures
2. Implement first game module (SWSH):
   - PokeRoutineExecutor8SWSH
   - TradeBotSWSH
3. Add USB connection support

### Phase 3: Discord Integration (Weeks 9-10)
1. Implement `sysbot_discord`:
   - Bot setup with serenity
   - Trade commands
   - Queue commands
   - Admin commands

### Phase 4: CLI Application (Week 11)
1. Implement `sysbot_cli`:
   - Config loading
   - Headless operation
   - Signal handling

### Phase 5: Additional Games (Weeks 12-14)
1. Add SV, BDSP, LA, LZA modules
2. Each follows SWSH pattern

### Phase 6: Secondary Integrations (Weeks 15-16)
1. Twitch bot
2. YouTube bot (if needed)
3. Z3 seed search

### Phase 7: GUI Application (Weeks 17-18)
1. Implement `sysbot_gui` with egui
2. Bot management UI
3. Configuration panels
4. Live logging

### Phase 8: Testing & Polish (Weeks 19-20)
1. Integration tests
2. Performance optimization
3. Documentation
4. Cross-platform testing

---

## 8. Test Strategy

### 8.1 Unit Tests

```rust
// tests/queue_tests.rs
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(81113333, 0, 8)]
    #[case(12345678, 3, 4)]
    fn test_get_digit(#[case] code: i32, #[case] digit: usize, #[case] expected: u8) {
        assert_eq!(TradeUtil::get_digit(code, digit), expected);
    }

    #[tokio::test]
    async fn test_enqueue_pk8() {
        let hub = TradeHub::<PK8>::new(Config::default());
        let trade = create_test_trade(1, false);

        let result = hub.queues.add_to_queue(trade, 1, false).await;
        assert!(matches!(result, QueueResult::Added { .. }));
    }
}
```

### 8.2 Integration Tests

```rust
// tests/integration/trade_flow.rs
#[tokio::test]
async fn test_full_trade_flow() {
    let mock_connection = MockSwitchConnection::new();
    let hub = setup_test_hub().await;
    let executor = SwshTradeBot::new(hub.clone(), mock_connection);

    // Add trade to queue
    let trade = create_trade_detail();
    hub.queues.add_to_queue(trade.clone(), 1, false).await;

    // Execute trade
    let result = executor.do_trade(&trade).await;
    assert!(matches!(result, TradeResult::Success));
}
```

### 8.3 Mock Implementations

```rust
// Mock connection for testing
pub struct MockSwitchConnection {
    responses: HashMap<Vec<u8>, Vec<u8>>,
}

#[async_trait]
impl SwitchConnection for MockSwitchConnection {
    async fn read_bytes(&self, offset: u32, length: usize) -> Result<Vec<u8>> {
        // Return predefined test data
        Ok(vec![0u8; length])
    }

    async fn write_bytes(&self, _data: &[u8], _offset: u32) -> Result<()> {
        Ok(())
    }
}
```

---

## 9. Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| PKHeX port complexity | HIGH | Start minimal, expand as needed |
| USB library issues | MEDIUM | Have rusb as fallback |
| Discord API changes | LOW | serenity is well-maintained |
| Performance regression | MEDIUM | Benchmark critical paths |
| Cross-platform GUI | MEDIUM | egui is proven cross-platform |

---

## 10. Benefits of Rust Migration

1. **Memory Safety** - No null pointer exceptions, buffer overflows
2. **Performance** - Zero-cost abstractions, no GC pauses
3. **Concurrency** - Fearless concurrency with ownership system
4. **Cross-Platform** - Single binary, no runtime dependency
5. **Modern Tooling** - Cargo, rustfmt, clippy
6. **Type System** - Algebraic data types, pattern matching
7. **Error Handling** - Result/Option types force explicit error handling

---

## 11. Estimated Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| Foundation | 4 weeks | sysbot_base + minimal pkhex_rs |
| Core Logic | 4 weeks | sysbot_pokemon + SWSH |
| Discord | 2 weeks | sysbot_discord |
| CLI | 1 week | sysbot_cli |
| Other Games | 3 weeks | SV, BDSP, LA, LZA |
| Secondary | 2 weeks | Twitch, YouTube, Z3 |
| GUI | 2 weeks | sysbot_gui |
| Testing | 2 weeks | Full test suite |
| **Total** | **20 weeks** | Complete migration |

---

## 12. File Reference

### Original C# Files to Migrate

**SysBot.Base (42 files)**
- Connection/Console/* → sysbot_base/connection/traits.rs
- Connection/Switch/Wireless/* → sysbot_base/connection/switch_socket.rs
- Connection/Switch/USB/* → sysbot_base/connection/switch_usb.rs
- Control/* → sysbot_base/control/*
- Util/* → sysbot_base/util/*
- Enums/* → sysbot_base/enums.rs

**SysBot.Pokemon (116 files)**
- TradeHub/* → sysbot_pokemon/trade_hub/*
- Queues/* → sysbot_pokemon/queues/*
- SWSH/* → sysbot_pokemon/games/swsh/*
- SV/* → sysbot_pokemon/games/sv/*
- (etc.)

**SysBot.Pokemon.Discord (36 files)**
- Commands/* → sysbot_discord/commands/*
- Helpers/* → sysbot_discord/helpers/*

---

## Conclusion

This migration is ambitious but achievable. The key success factors are:

1. **Start with minimal PKHeX port** - Don't try to port everything
2. **Use Tokio consistently** - Single async runtime
3. **Leverage Rust's type system** - Generic traits over inheritance
4. **Test incrementally** - Each module before moving on
5. **Cross-platform from day 1** - Avoid platform-specific code

The resulting Rust codebase will be faster, safer, and more maintainable than the C# original.
