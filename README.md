# SysBot.rs 🦀

A Rust implementation of Nintendo Switch Pokemon automation with a REST API.

![License](https://img.shields.io/badge/License-AGPLv3-blue.svg)
![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)

## Features

- **API-First Design**: REST API that any client can consume (Discord bots, web apps, mobile apps)
- **No Platform Lock-in**: Not tied to Discord, Twitch, or any specific platform
- **Fast & Safe**: Built in Rust with async/await
- **78 Tests**: Comprehensive test coverage with TDD approach

---

## 📋 Prerequisites

### 1. Nintendo Switch Setup

Your Nintendo Switch needs **Custom Firmware (CFW)** with **sys-botbase** installed.

#### Required:
- Nintendo Switch with CFW (Atmosphere recommended)
- [sys-botbase](https://github.com/olliz0r/sys-botbase) v2.4+ installed
- Switch and computer on the same network

#### Installing sys-botbase:

1. Download the latest release from [sys-botbase releases](https://github.com/olliz0r/sys-botbase/releases)
2. Extract and copy the `atmosphere` folder to your SD card root
3. Reboot your Switch

### 2. Development Environment

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

---

## 🚀 Quick Start

### Step 1: Clone and Build

```bash
git clone https://github.com/0xS4D/SysBot.NET.git
cd SysBot.NET
cargo build --release
```

### Step 2: Find Your Switch IP Address

On your Nintendo Switch:
1. Go to **System Settings** → **Internet** → **Connection Status**
2. Note the **IP Address** (e.g., `192.168.1.100`)

### Step 3: Start the API Server

```bash
cargo run -p sysbot_api
```

The server starts on `http://localhost:3000`

### Step 4: Connect to Your Switch

```bash
# Add a bot (replace with your Switch IP)
curl -X POST http://localhost:3000/api/v1/bots \
  -H "Content-Type: application/json" \
  -d '{"id": "switch1", "ip": "192.168.1.100", "port": 6000}'

# Connect to the Switch
curl -X POST http://localhost:3000/api/v1/bots/switch1/connect

# Verify connection by getting sys-botbase version
curl http://localhost:3000/api/v1/bots/switch1/version
```

---

## 🧪 Testing Connection

### Run Unit Tests

```bash
cargo test
```

### Run Connection Integration Test

Create a `.env` file with your Switch IP:

```bash
echo "SWITCH_IP=192.168.1.100" > .env
```

Then run the integration tests:

```bash
cargo test --test integration_tests -- --ignored
```

Or test manually with the API:

```bash
# 1. Start the server
cargo run -p sysbot_api &

# 2. Add and connect bot
curl -X POST http://localhost:3000/api/v1/bots \
  -H "Content-Type: application/json" \
  -d '{"id": "test", "ip": "YOUR_SWITCH_IP", "port": 6000}'

curl -X POST http://localhost:3000/api/v1/bots/test/connect

# 3. Test button click (press A button)
curl -X POST http://localhost:3000/api/v1/bots/test/click \
  -H "Content-Type: application/json" \
  -d '{"button": "A"}'

# 4. Get game title ID
curl http://localhost:3000/api/v1/bots/test/title

# 5. Read memory (example: read 4 bytes from heap offset 0x1000)
curl -X POST http://localhost:3000/api/v1/bots/test/peek \
  -H "Content-Type: application/json" \
  -d '{"offset": 4096, "size": 4, "offset_type": "heap"}'
```

---

## 📡 API Endpoints

### Health
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Health check |

### Bot Management
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/bots` | List all bots |
| POST | `/api/v1/bots` | Add a new bot |
| GET | `/api/v1/bots/{id}` | Get bot info |
| DELETE | `/api/v1/bots/{id}` | Remove a bot |
| POST | `/api/v1/bots/{id}/connect` | Connect to Switch |
| POST | `/api/v1/bots/{id}/disconnect` | Disconnect from Switch |

### Controls
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/bots/{id}/click` | Click a button |
| POST | `/api/v1/bots/{id}/press` | Press and hold |
| POST | `/api/v1/bots/{id}/release` | Release button |
| POST | `/api/v1/bots/{id}/stick` | Set stick position |

### Memory
| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/bots/{id}/peek` | Read memory |
| POST | `/api/v1/bots/{id}/poke` | Write memory |

### System
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/bots/{id}/title` | Get game title ID |
| GET | `/api/v1/bots/{id}/version` | Get sys-botbase version |

---

## 🎮 Button Names

Available buttons for click/press/release:
- `A`, `B`, `X`, `Y`
- `L`, `R`, `ZL`, `ZR`
- `PLUS` (or `+`), `MINUS` (or `-`)
- `DUP`, `DDOWN`, `DLEFT`, `DRIGHT` (or `UP`, `DOWN`, `LEFT`, `RIGHT`)
- `LSTICK`, `RSTICK`
- `HOME`, `CAPTURE`

Sticks: `LEFT` (or `L`), `RIGHT` (or `R`)

---

## 🔧 Troubleshooting

### "Connection refused" error

1. Make sure sys-botbase is running on your Switch
2. Verify your Switch IP address is correct
3. Check that both devices are on the same network
4. Ensure port 6000 is not blocked by firewall

### "Timeout" error

1. sys-botbase might not be responding
2. Try rebooting your Switch
3. Check if a game is running (some commands require a game)

### "Invalid response" error

1. sys-botbase version might be outdated
2. Update to sys-botbase v2.4 or newer

---

## 📁 Project Structure

```
.
├── Cargo.toml              # Workspace configuration
├── crates/
│   ├── sysbot_base/        # Core library (connection, commands)
│   │   └── src/
│   │       ├── command.rs      # Button, stick, memory commands
│   │       ├── connection/     # WiFi/USB connection implementations
│   │       └── error.rs        # Error types
│   │
│   └── sysbot_api/         # REST API
│       └── src/
│           ├── main.rs         # Server entry point
│           ├── routes.rs       # API endpoints
│           └── state.rs        # Application state
└── tests/                  # Integration tests
```

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

---

## 📜 License

This project is licensed under AGPLv3 - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Credits

- [sys-botbase](https://github.com/olliz0r/sys-botbase) - The foundation that makes Switch automation possible
- [PKHeX](https://github.com/kwsch/PKHeX) - Pokemon data structures and legality checking
- Original [SysBot.NET](https://github.com/kwsch/SysBot.NET) - Inspiration for this project
