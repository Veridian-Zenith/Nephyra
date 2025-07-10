---

# 🧠 Nephyra (Beta/Test)

**Nephyra** is your terminal-based, smart system insight assistant.
She’ll tell you everything you didn’t even know you needed.

> *Status: Work in Progress*

---

## 🎯 Goals

* Cross-check active vs installed kernel versions
* Detect mismatched headers and kernel modules
* Report system performance, boot mode, DE/WM information
* Natural-language input and output for commands
* Many more features to come — still in early development

---

## 🛠️ Building Nephyra

Build scripts are provided for common shells. The compiled binary will be placed in `~/bin` — ensure that directory is in your `$PATH`.

### For Bash, sh, or Zsh:

```sh
./build.sh
# or
./build.zsh
```

### For Fish:

```fish
./build.fish
```

The build script compiles the release binary and places it in `~/bin/Nephyra`.

---

## 🧭 PATH Setup

To run `Nephyra` globally from anywhere, add `~/bin` to your shell's `$PATH`.

### Temporary (Current Session Only)

#### Bash/sh/Zsh

```sh
export PATH="$HOME/bin:$PATH"
```

#### Fish

```fish
set -gx PATH $HOME/bin $PATH
```

---

### Permanent (All Future Sessions)

#### Bash

Add to your `~/.bashrc` or `~/.bash_profile`:

```sh
export PATH="$HOME/bin:$PATH"
```

#### Zsh

Add to your `~/.zshrc`:

```sh
export PATH="$HOME/bin:$PATH"
```

#### Fish

Add to `~/.config/fish/config.fish`:

```fish
fish_add_path $HOME/bin
```

Then restart your terminal or run `source` on the appropriate config file to apply changes.

---

## 🚀 Usage

Run Nephyra by passing a module name:

```sh
Nephyra kernel
Nephyra bootloader
Nephyra hardware         # ⚠️ Dumps detailed info to hardware_info.log
Nephyra power
Nephyra report
Nephyra packages         # Checks for orphaned/outdated packages + update manager
```

### Notes:

* `packages` module supports: **pacman**, **apt**, **dnf**, **apk**, **zypper**, and **emerge**.
* If orphaned packages are found, you’ll be prompted (requires `sudo`) to clean them up.
* The `bootloader` module **may require root privileges** on some systems.

### Running Without PATH Setup:

You can always run directly from the build output directory:

```sh
./target/release/Nephyra report
```

---

## 🤝 Contributing

PRs, issues, and suggestions are warmly welcomed!
Nephyra is still early in development, so ideas and feedback are appreciated.

---

## 📝 License

**Dual-licensed** under the **GNU AGPLv3** for community use and the **Veridian Commercial License (VCL 1.0)** for proprietary applications.

See the [LICENSE](LICENSE) file for full details.

---

## ⚖️ Legal Disclaimer

**Veridian Zenith** is a digital label and project organization operated by **Jeremy Matlock**, also known as **Dae Euhwa**.
All works published under this name are the intellectual property of Jeremy Matlock unless otherwise stated.

---

© 2025 Veridian Zenith

---
