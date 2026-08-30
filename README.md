# 🦎 Chameleon

The project is under development. Nothing (almost) is working yet

## Launcher

Just launcher with fuzzy search

![launcher](img/launcher.png)

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

```lua
bind(main_mod .. "+ RETURN", hl.dsp.exec_raw("~/.chameleon/bin/chameleon-launcher -t"))
```

## Panel

![panel](img/panel.png)
