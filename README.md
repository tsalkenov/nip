# Nip

Wrapper for nix-shell which adds persistence(protection from nix gc)

## Installation

You can install nip by adding nip flake to your nixos configuration

```nix
# flake.nix

{
  inputs.nip.url = "github:tsalkenov/nip"; # Add flake as input

  outputs = {nixpkgs, ...} @ inputs: {
    nixosConfigurations.HOSTNAME = nixpkgs.lib.nixosSystem {
      specialArgs = { inherit inputs; }; # This is the important part
      modules = [
        ./configuration.nix
      ];
    };
  } 
}

# configuration.nix

{inputs, pkgs, ...}: {
  environment.systemPackages = with pkgs; [
    inputs.nip.packages.${system}.default # Add nip to package list
    # Other packages
  ]
}
```

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
