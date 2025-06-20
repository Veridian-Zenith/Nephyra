# Nephyra (Beta/Test)

Nephyra is your terminal-based, smart system insight assistant.

And she’ll tell you everything you didn’t know you needed. (WIP)

## 🔧 Goals
- Cross-check active vs installed kernels
- Detect mismatched headers/modules
- Report system performance, boot mode, DE/WM info
- Natural-language input/output
- More to come, still a WIP project.

## How to build

You can build Nephyra using the provided scripts for your shell. The binary will be placed in your `~/bin` directory (make sure it's in your `$PATH`):

### For bash/sh/zsh
```sh
./build.sh
# or
./build.zsh
```

### For fish shell
```fish
./build.fish
```

This will build the release version and copy the binary to `~/bin/Nephyra`.

## Path Setup

To run `Nephyra` from anywhere, add `~/bin` to your `PATH`.

### Temporary (for current session only)

#### Bash/sh/zsh
```sh
export PATH="$HOME/bin:$PATH"
```

#### fish
```fish
set -gx PATH $HOME/bin $PATH
```

### Permanent (all future sessions)

#### Bash
Add this to your `~/.bashrc` or `~/.bash_profile`:
```sh
export PATH="$HOME/bin:$PATH"
```

#### Zsh
Add this to your `~/.zshrc`:
```sh
PATH="$HOME/bin:$PATH"
```

#### fish
Add this to your `~/.config/fish/config.fish`:
```fish
fish_add_path $HOME/bin
```

After updating your config, restart your terminal or run the relevant `source` command to reload your shell configuration.

## Usage


Run Nephyra with a module name:

```sh
Nephyra kernel
Nephyra bootloader
Nephyra hardware
Nephyra power
```

Note: The bootloader command **likely requires root privileges**.

You can also run the binary directly from the build directory:

```sh
./target/release/Nephyra kernel
```

## Contributing
PRs and issues welcome! This is a work in progress.

---

© 2025 Nephyra contributors
