# Flake's devShell for non-flake-enabled nix instances
let
  compat = (import ./compat.nix { src = ./.; });
in

compat.defaultNix.default
