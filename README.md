# q - AI-Powered Terminal Command Assistant

`q` is a terminal helper that uses AI to interpret natural language queries and suggest (or execute) appropriate shell commands.

## Installation

### Quick Start

1. **Install q**:
   ```bash
   cargo install q-cli
   ```

2. **Configure your API key**:

   ```bash
   q config
   ```

## Usage

### Basic Usage

Simply ask `q` what you want to do in natural language:

```bash
q find all files larger than 100MB
q "list all git branches with 'feature' in the name"
q show me the 10 largest directories
q kill the process using port 8080
```

### Command-Line Options

- `--config-path`: Show the location of the config file
- `-y, --yes`: Skip confirmation and execute immediately (use with caution!)

### Configuration

The config file is located at:
- **Windows**: `%APPDATA%\q\config.toml`
- **macOS**: `~/Library/Application Support/q/config.toml`
- **Linux**: `~/.config/q/config.toml` (or in `$XDG_CONFIG_HOME` if set)

## Future Enhancements

- [ ] Support for additional AI providers (OpenAI, Anthropic, local LLMs)
- [ ] Command history and favorites
- [ ] Multi-step command sequences
- [ ] Interactive command editing before execution
- [ ] Shell integration for better context awareness

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
