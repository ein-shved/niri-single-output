# Niri single output

This is utility for [niri compositor](https://github.com/YaLTeR/niri) useful in
setups, where you may have several outputs connected to your PC, but wants to
have only once active output at a time.

## Building

### Cargo

You can use `cargo` to build this project

### Nix

This project has flake files which allows you to build it as NIX package with

```bash
nix build
```

## Usage

```
niri-single-output [OPTIONS] <COMMAND>
```

### Commands

 * `test`  Check niri availability
 * `init`  Init outputs at startup
 * `next`  Switch to next output

### Options:
  * `-p`, `--path` `<PATH>` Path to niri socket
  * `-s`, `--state` `<STATE>` Path to niri socket
  * `-h`, `--help` Print help (see a summary with '-h')
  * `-V`, `--version` Print version

## Application

This utility attended to be use from niri configuration.

Spawn the `init` command at startup:

```kdl
spawn-at-startup "niri-single-output" "init"
```

Add keybinding to switch to next output:

```kdl
binds {
    Mod+O { spawn "niri-single-output" "next"; }
}
```
