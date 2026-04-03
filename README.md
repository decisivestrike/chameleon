# 🦎 Chameleon

The project is under development. Nothing (almost) is working yet

## Launcher

Just launcher with fuzzy search

```toml
[launcher]
enabled = true
placeholder = "let's find something..."

# for opening terminal apps
terminal_cmd = "kitty"

# child process detach
detach = true
```

To use it in hyprland:

```bash
exec-once = chameleon -c ~/.config/chameleon/stable-config.toml

$launcher = chameleon -t
bind = $mainMod, RETURN, exec, $launcher
```
