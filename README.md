# NIP

Wrapper for nix-shell which adds persistence(protection from nix gc)

## Usage

Start nix-shell in current directory and save it:

```bash
nip # Yeah that's all
```

Running shell in another directory:

```bash
nip /directory/with/shell
```

## Supported shells

### Impure nix shells (shell.nix/default.nix)

First class support

### Flaky dev shells (flake.nix)

Not supported yet
