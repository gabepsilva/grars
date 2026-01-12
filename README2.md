# Insight Reader TTS Engine

<div align="center">
<img src="assets/logo.svg" height="128">

![macOS](https://img.shields.io/badge/macOS-Compatible-black?style=for-the-badge&logo=apple)
![Linux](https://img.shields.io/badge/Linux-Compatible-black?style=for-the-badge&logo=linux)
![Rust](https://img.shields.io/badge/Rust-2021+-orange?style=for-the-badge&logo=rust)
![Piper](https://img.shields.io/badge/Piper-TTS-green?style=for-the-badge)
![AWS Polly](https://img.shields.io/badge/AWS-Polly-yellow?style=for-the-badge)



*High-quality text-to-speech with beautiful GUI, multiple providers, and cross-platform support*

</div>

## ✨ Features

<table>
<tr>
<td width="50%">

**🌍 Multiple TTS Providers**
- **Piper** (local, offline) - Fast, privacy-focused local TTS with 100+ voices
- **AWS Polly** (cloud) - High-quality neural voices with multiple engines (Standard, Neural, Generative, LongForm)

**🎨 Modern GUI**
- Floating borderless window with drag support
- Real-time waveform visualization
- Play/pause/stop controls
- Skip forward/backward (5 seconds)

</td>
<td width="50%">

**🎯 System Integration**
- Works with any application (browser, editor, etc.)
- Cross-platform support (Linux, macOS)
- Text cleanup toggle


**⚡ Lightning Fast**
- Native Rust performance
- Streaming audio playback
- Low latency audio synthesis

**🔊 High Quality**
- Super high quality neural audio synthesis
- Multiple voice engines (Standard, Neural, Generative, LongForm for AWS Polly)


</td>
</tr>
</table>

## 🚀 Easy installation

```bash
curl -fsSL https://insightreader.xyz/install.sh | bash
```

## 📸 Screenshots

<div align="center">

### Main Window
<img src="assets/screenshots/main.png" alt="Main Window" width="300">

*Floating borderless window with waveform visualization and playback controls*

### Settings Window
<img src="assets/screenshots/configurations.png" alt="Settings Window" width="600">

*Comprehensive settings with provider selection, voice management, and configuration options*

</div>

## 🗣️ Available Voices & Languages


**100+ voices** Offline voices by [rhasspy/piper-voices](https://huggingface.co/rhasspy/piper-voices) repository:


- **Low** - Fast, smaller models
- **Medium** - Balanced quality and speed
- **High** - Best quality, larger models

Voices can be downloaded directly from the application interface.

**100+ neural voices** across **30+ languages** by AWS Polly:

- **Standard** - Traditional TTS
- **Neural** - High-quality neural voices
- **Generative** - Advanced AI-generated voices
- **LongForm** - Optimized for long-form content


### 🚀 Installation

#### Quick Install (Recommended)

Run the installation script directly:

```bash
curl -fsSL https://insightreader.xyz/install.sh | bash
```

Or using `wget`:

```bash
wget -qO- https://insightreader.xyz/install.sh | bash
```

#### Manual Installation

1. **Prerequisites**:
   - **Rust** (latest stable version)
   - **System dependencies**:
     - `python3` and `python3-venv` (for Piper TTS)
     - `espeak-ng` (for text processing)

2. **Clone and build**:
   ```bash
   git clone https://github.com/gabepsilva/insight-reader.git
   cd insight-reader
   cargo build --release
   ```

3. **Install the binary**:
   ```bash
   cp target/release/insight-reader ~/.local/bin/
   ```

4. **Set up Piper TTS** (for local TTS):
   ```bash
   mkdir -p ~/.local/share/insight-reader/venv
   python3 -m venv ~/.local/share/insight-reader/venv
   source ~/.local/share/insight-reader/venv/bin/activate
   pip install piper-tts
   ```

5. **Download a Piper voice model** (optional - can be done from UI):
   ```bash
   mkdir -p ~/.local/share/insight-reader/models
   # Download from https://huggingface.co/rhasspy/piper-voices
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

## 🎯 Usage

### Basic Usage

1. **Select text** in any application (browser, editor, etc.)
2. **Run Insight Reader**:
3. The application will:
   - Read the selected text automatically
   - Display a floating window
   - Start speaking immediately


## 🔧 Advanced Usage


### Text Cleanup

Enable text cleanup in settings to:
- Remove markdown formatting
- Clean up special characters
- Improve TTS quality for formatted text

## 🛠️ Troubleshooting

### Common Issues

**"No audio playback"**
- Check that your system audio is working

**"AWS Polly not working"**
- Verify AWS credentials are configured (see [AWS Polly Setup](#aws-polly-setup-optional))
- Check error messages in the settings window
- Verify AWS credentials have Polly permissions

**"Clipboard not working"**
- **macOS**: 
  - Grant accessibility permissions: **System Preferences/Settings → Security & Privacy → Privacy → Accessibility**
  - Add Insight Reader (or Terminal if running from terminal) to the allowed apps list
  - Try selecting text before running Insight Reader


```

## 🗺️ Roadmap

- [x] Multiple TTS providers (Piper, AWS Polly)
- [x] Beautiful floating GUI with Iced
- [x] Real-time waveform visualization
- [x] Voice selection for both providers
- [x] Voice download from UI
- [x] Text cleanup support
- [x] Cross-platform support (Linux, macOS)
- [x] Settings persistence
- [ ] Windows support
- [ ] Read Images
- [ ] Batch text processing
- [ ] Audio export functionality
- [ ] Plugin system for custom providers

## 🤝 Contributing

We welcome contributions! Please feel free to:
- Report bugs and issues
- Suggest new features
- Submit pull requests
- Add new TTS providers
- Improve documentation
- Add new voice packs
- Design UI/UX improvements

## 📊 Tested Platforms

**Release tested:**

| Platform | Desktop Environment / WM | Status |
|----------|------------------------|--------|
| 🐧 **Ubuntu** | 🖥️ GNOME (Wayland) | ✅ Tested |
| 🐧 **Ubuntu** | 🎨 KDE (Wayland/X11) | ✅ Tested |
| 🎩 **Fedora** | 🖥️ GNOME (Wayland) | ✅ Tested |
| 🎩 **Fedora** | 🎨 KDE (Wayland/X11) | ✅ Tested |
| 🏛️ **Arch Linux** | 🖥️ GNOME (Wayland) | ✅ Tested |
| 🏛️ **Arch Linux** | 🎨 KDE (Wayland/X11) | ✅ Tested |
| 🏛️ **Arch Linux** | 🌊 Hyprland (Wayland) | ✅ Tested |
| 🍎 **macOS** | Apple Silicon (M1/M2/M3) | ✅ Tested |
| 🍎 **macOS** | Intel | ✅ Tested |

While it should work on other Linux distributions and window managers, these are the primary tested environments.

## 📝 Logging

Logs are written to:
- **Stderr**: Real-time console output
- **File**: `~/.local/share/insight-reader/logs/insight-reader-YYYY-MM-DD.log`


## 🙏 Acknowledgments

- Built with [Iced](https://iced.rs/) GUI framework
- Uses [Piper TTS](https://github.com/rhasspy/piper) for local TTS
- AWS Polly integration via AWS SDK for Rust
- Audio playback powered by [rodio](https://github.com/RustAudio/rodio)
- Waveform visualization using [rustfft](https://github.com/ejmahler/rustfft)

---

<div align="center">

**Made with ❤️ for the open-source community**

[GitHub](https://github.com/gabepsilva/insight-reader) • [Issues](https://github.com/gabepsilva/insight-reader/issues) • [Releases](https://github.com/gabepsilva/insight-reader/releases)

</div>
