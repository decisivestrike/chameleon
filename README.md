# 🦎 Chameleon

The project is under development. Nothing (almost) is working yet

Installation:

```bash
curl -fsSL https://raw.githubusercontent.com/decisivestrike/chameleon/main/scripts/install.sh | bash
```

## Launcher

Just launcher with fuzzy search

```toml
# launcher.toml

enabled = true
placeholder = "let's find something..."

# for opening terminal apps
terminal_cmd = "kitty"

# child process detach
detach = true
```

To use it in hyprland:

```bash
# Add to autostart
exec-once = chameleon

# Create alias
$launcher = chameleon -t

# Bind keyboard shortcut
bind = $mainMod, RETURN, exec, $launcher
```
