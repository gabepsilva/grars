# Insight Reader

A modern, lightweight Text-to-Speech (TTS) application written in Rust with Iced GUI. Reads any selected text on your computer and speaks it aloud with a beautiful floating window interface.

**🚀 Get started in seconds:** `curl -fsSL https://insightreader.xyz/install.sh | bash`

## Features

- 🎤 **Multiple TTS Providers**
  - **Piper** (local, offline) - Fast, privacy-focused local TTS
  - **AWS Polly** (cloud) - High-quality neural voices

- 🎨 **Modern UI**
  - Floating borderless window
  - Real-time waveform visualization (FFT-driven)
  - Dynamic progress bar
  - Play/pause/stop controls
  - Skip forward/backward (5 seconds)

- ⚙️ **Settings & Configuration**
  - Provider selection (Piper/Polly)
  - Log level configuration
  - Persistent settings (saved to `~/.config/insight-reader/config.json`)

- 🔧 **Additional Features**
  - Automatic clipboard/selection reading at startup
  - Auto-close window when playback completes
  - Error handling with user-friendly messages
  - Comprehensive logging

## Screenshots

The application displays a compact floating window with:
- Volume icon and animated waveform bars
- Playback controls (-5s, +5s, play/pause, stop)
- Progress bar
- Settings gear icon

## Tested Platforms

This application has been tested on:
- **Ubuntu** with GNOME (Wayland)
- **Arch Linux** with Hyprland (Wayland)
- **macOS** (Apple Silicon and Intel)

While it should work on other Linux distributions and window managers, these are the primary tested environments.

## Installation

### Quick Install (Recommended)

Run the installation script directly from GitHub:

```bash
curl -fsSL https://insightreader.xyz/install.sh | bash
```

Or using `wget`:

```bash
wget -qO- https://insightreader.xyz/install.sh | bash
```

### Manual Installation

#### Prerequisites

- **Rust** (latest stable version)
- **System dependencies** (automatically installed by install script):
  - `python3` and `python3-venv` (for Piper TTS)
  - `espeak-ng` (for text processing)
  - **Linux**: No additional dependencies (clipboard handled by `arboard` crate)
  - **macOS**: No additional dependencies (clipboard handled by `arboard` crate)

#### Build from Source

1. Clone the repository:
   ```bash
   git clone https://github.com/gabepsilva/insight-reader.git
   cd insight-reader
   ```

2. Build the project:
   ```bash
   cargo build --release
   ```

3. Install the binary:
   ```bash
   cp target/release/insight-reader ~/.local/bin/
   ```

4. Set up Piper TTS (for local TTS):
   ```bash
   # The install.sh script handles this automatically, but manually:
   mkdir -p ~/.local/share/insight-reader/venv
   python3 -m venv ~/.local/share/insight-reader/venv
   source ~/.local/share/insight-reader/venv/bin/activate
   pip install piper-tts
   ```

5. Download a Piper voice model:
   ```bash
   mkdir -p ~/.local/share/insight-reader/models
   # Download a model from https://huggingface.co/rhasspy/piper-voices
   # Example: en_US-lessac-medium
   ```

#### AWS Polly Setup (Optional)

To use AWS Polly, configure your AWS credentials:

1. **Environment variables** (recommended):
   ```bash
   export AWS_ACCESS_KEY_ID="your-access-key"
   export AWS_SECRET_ACCESS_KEY="your-secret-key"
   export AWS_REGION="us-east-1"  # optional, defaults to us-east-1
   ```

2. **Or credentials file** (`~/.aws/credentials`):
   ```ini
   [default]
   aws_access_key_id = your-access-key
   aws_secret_access_key = your-secret-key
   ```

3. **Or named profile** (`~/.aws/credentials`):
   ```ini
   [profile myprofile]
   aws_access_key_id = your-access-key
   aws_secret_access_key = your-secret-key
   ```
   Then set: `export AWS_PROFILE=myprofile`

## Usage

### Basic Usage

1. **Select text** in any application (browser, editor, etc.)
2. **Run Insight Reader**:
   ```bash
   insight-reader
   ```
3. The application will:
   - Read the selected text automatically
   - Display a floating window
   - Start speaking immediately

### Controls

- **-5s / +5s**: Skip backward/forward by 5 seconds
- **Play/Pause**: Toggle playback
- **Stop**: Stop playback and close window
- **Settings (⚙)**: Open settings window to change provider or log level

### Settings

Click the settings gear icon to:
- Switch between Piper (local) and AWS Polly (cloud) providers
- Adjust log level (Error, Warn, Info, Debug, Trace)
- View error messages if any issues occur

Settings are automatically saved to `~/.config/insight-reader/config.json`.

## Project Structure

```
src/
├── main.rs              # Entry point
├── app.rs               # Iced Application (async-capable)
├── model.rs             # Domain types + app state
├── update.rs            # Business logic
├── view.rs              # UI rendering
├── styles.rs            # Custom styles
├── config.rs            # Configuration management
├── logging.rs           # Logging setup
├── providers/
│   ├── mod.rs           # TTSProvider trait
│   ├── piper.rs         # Piper TTS implementation
│   ├── polly.rs         # AWS Polly implementation
│   └── audio_player.rs  # Audio playback engine
└── system/
    ├── mod.rs
    └── clipboard.rs     # Clipboard/selection reading
```

## Configuration

Configuration is stored in `~/.config/insight-reader/config.json`:

```json
{
  "voice_provider": "piper",
  "log_level": "INFO"
}
```

Valid values:
- `voice_provider`: `"piper"` or `"polly"`
- `log_level`: `"ERROR"`, `"WARN"`, `"INFO"`, `"DEBUG"`, `"TRACE"`

## Logging

Logs are written to:
- **Stderr**: Real-time console output
- **File**: `~/.local/share/insight-reader/logs/insight-reader-YYYY-MM-DD.log`

Log level can be changed in the settings window.

## Troubleshooting

### No audio playback

- Check that your system audio is working
- Verify the TTS provider is correctly configured
- Check logs: `~/.local/share/insight-reader/logs/insight-reader-*.log`

### AWS Polly not working

- Verify AWS credentials are configured (see [AWS Polly Setup](#aws-polly-setup-optional))
- Check error messages in the settings window
- Ensure you have internet connectivity
- Verify AWS credentials have Polly permissions

### Clipboard not working

- **Linux/macOS**: Clipboard support is handled by the `arboard` crate (no external dependencies needed)
- **macOS**: 
  - Grant accessibility permissions: **System Preferences/Settings → Security & Privacy → Privacy → Accessibility**
  - Add Insight Reader (or Terminal if running from terminal) to the allowed apps list
  - Try selecting text before running Insight Reader
- Try selecting text before running Insight Reader

### Piper TTS not found

- Ensure Python venv is set up: `~/.local/share/insight-reader/venv`
- Verify `piper-tts` is installed in the venv
- Check that a voice model is downloaded to `~/.local/share/insight-reader/models`

## Development

### Building

```bash
cargo build
```

### Running

```bash
cargo run
```

### Testing

```bash
cargo test
```

### Dependencies

Key dependencies:
- `iced` - GUI framework
- `rodio` - Audio playback
- `rustfft` - FFT for waveform visualization
- `aws-sdk-polly` - AWS Polly integration
- `tokio` - Async runtime
- `tracing` - Structured logging

See `Cargo.toml` for the complete list.

## License

[Add your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- Built with [Iced](https://iced.rs/) GUI framework
- Uses [Piper TTS](https://github.com/rhasspy/piper) for local TTS
- AWS Polly integration via AWS SDK for Rust

