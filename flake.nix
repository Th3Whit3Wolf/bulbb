{

  inputs = {
    nixpkgs.url = "nixpkgs/release-21.05";
    utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
    naersk.url = "github:nix-community/naersk";
    devshell.url = "github:numtide/devshell";
  };

  outputs = { self, utils, rust-overlay, devshell, nixpkgs, naersk, ... }:
    utils.lib.eachDefaultSystem (system:
      let
        inherit (naersk.lib.${system}) buildPackage;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ devshell.overlay rust-overlay.overlay ];
        };
        rust-stable = pkgs.rust-bin.stable."1.56.1".default.override {
          extensions = [
            "cargo"
            "clippy"
            "rust-docs"
            "rust-src"
            "rust-std"
            "rustc"
            "rustfmt"
          ];
        };

        naersk-lib = (naersk.lib."${system}".override {
          cargo = rust-stable;
          rustc = rust-stable;
        });

        codeSettings = pkgs.writeScriptBin "vscode-settings" ''
          #!${pkgs.stdenv.shell}

          if [ ! -f "''${PRJ_ROOT}/.vscode/settings.json" ]; then
            if [ ! -d "''${PRJ_ROOT}/.vscode" ]; then
              mkdir "''${PRJ_ROOT}/.vscode"
            fi
  
          ${pkgs.coreutils}/bin/cat <<EOF > $PRJ_ROOT/.vscode/settings.json
          {
              "rust-analyzer.trace.extension": true,
              "rust-analyzer.trace.server": "messages",
              "rust-analyzer.server.path": "${pkgs.rust-analyzer}/bin/rust-analyzer",
              "terminal.integrated.profiles.linux": {
                  "bash": {
                      "path": "bash"
                  },
                  "zsh": {
                      "path": "zsh"
                  },
                  "nix" : {
                      "path": "nix-shell"
                  }
              },
              "terminal.integrated.defaultProfile.linux": "nix",
              "editor.insertSpaces": false,
          }
          EOF

          else
            line=$(grep '"rust-analyzer.server.path"' $PRJ_ROOT/.vscode/settings.json)
            nline='"rust-analyzer.server.path": "${pkgs.rust-analyzer}/bin/rust-analyzer",'
            sed -i "s|$line|$nline|g" $PRJ_ROOT/.vscode/settings.json
          fi
        '';

        newFile = pkgs.writeScriptBin "newFile" ''
          #!${pkgs.stdenv.shell}

          set -Eeo pipefail
          trap cleanup SIGINT SIGTERM ERR EXIT

          ROOT_PATH="''${PRJ_ROOT}/src"

          usage() {
            ${pkgs.coreutils}/bin/cat <<EOF
          Usage: new [-h] [-d] PATH

          Create a new file with a license header in it.

          Available options:

          -h, --help      Print this help and exit
          -d, --directory Create a directory with a mod.rs inside of it
          EOF
            exit
          }

          cleanup() {
            trap - SIGINT SIGTERM ERR EXIT
            # script cleanup here
          }

          setup_colors() {
            if [[ -t 2 ]] && [[ -z "''${NO_COLOR-}" ]] && [[ "''${TERM-}" != "dumb" ]]; then
              NOFORMAT='\033[0m' RED='\033[0;31m' GREEN='\033[0;32m' ORANGE='\033[0;33m' BLUE='\033[0;34m' PURPLE='\033[0;35m' CYAN='\033[0;36m' YELLOW='\033[1;33m'
            else
              NOFORMAT="" RED="" GREEN="" ORANGE="" BLUE="" PURPLE="" CYAN="" YELLOW=""
            fi
          }

          msg() {
            echo >&2 -e "''${1-}"
          }

          die() {
            local msg=$1
            local code=''${2-1} # default exit status 1
            msg "$msg"
            exit "$code"
          }

          cleanse_path() {
            pth=$1
            if [[ "''${pth:0:1}" == "/" || "''${pth:0:1}" == "~" || "''${pth:0:1}" == "$" ]]; then
              die "Please give path relative to ''${ROOT_PATH}"
            else
              echo "''${ROOT_PATH}/''${pth}"
            fi
          }

          get_rel_path() {
            PATH=$1
            REL=$(${pkgs.coreutils}/bin/realpath --relative-to="''${PRJ_ROOT}" "''${PATH}")
            echo "''${REL}"
          }

          mkFile() {
            PATH=$(cleanse_path $1)
            DIR=$(${pkgs.coreutils}/bin/dirname "''${PATH}")
            FILENAME=$(${pkgs.coreutils}/bin/basename "''${PATH}")
            REL_DIR=$(get_rel_path ''${DIR})
            REL_FILE=$(get_rel_path ''${PATH})

            if [[ ! -d "''${DIR}" ]]; then
              mkdir -p "''${DIR}"
              echo "Creatied directory ''${REL_DIR}"
            fi
            if [[ "''${FILENAME}" == *"."* ]]; then 
              FILE_EXTENSION=$(echo "''${FILENAME}" | cut -d '.' -f2- )

              case "''${FILE_EXTENSION}" in
                rs)
                  ${pkgs.coreutils}/bin/cat <<EOF >> "''${PATH}"
          /*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
          */


          EOF
          echo "Created file ''${REL_FILE}"

              ;;
              *) die "Unrecongnized file extension" ;;
              esac
            else
              ${pkgs.coreutils}/bin/cat <<EOF >> "''${PATH}.rs"
          /*
Copyright 2021 David Karrick

Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
<LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
option. This file may not be copied, modified, or distributed
except according to those terms.
          */


          EOF
          echo "Created file ''${REL_FILE}.rs"
            fi
          }

          main() {
            args=("$@")

            if [[ ''${#args[@]} -gt 0 ]]; then
              while :; do
                case "''${args[0]}" in
                -h | --help) usage ;;
                -d | --directory) 
                  if [[ ''${#args[@]} -eq 2 ]]; then
                    mkFile "''${args[1]}/mod.rs"
          
                    return 0
                  else
                    die "Missing path to new directory"
                  fi
                ;;
                *)
                  mkFile "''${args[0]}"
        
                  return 0
                ;;
                esac
              done
            else
              die "Missing script arguments"
            fi
          }

          main "$@"
          setup_colors

        '';

        extensions = (with pkgs.vscode-extensions; [
          bbenoist.Nix
          matklad.rust-analyzer
          tamasfe.even-better-toml
          pkief.material-icon-theme
        ]) ++ pkgs.vscode-utils.extensionsFromVscodeMarketplace [
          {
            name = "spacemacs";
            publisher = "cometeer";
            version = "1.1.1";
            sha256 =
              "da54d2a40b72bb814b2e4af6b03eff6b3982162ae6f4492e6ceccad8f70cc7d3";
          }
          {
            name = "search-crates-io";
            publisher = "belfz";
            version = "1.2.1";
            sha256 =
              "2b61f83871fabe042f86170e15d3f7443d1f3e0840c716e0babbfe37cda914db";
          }
          {
            name = "errorlens";
            publisher = "usernamehw";
            version = "3.4.0";
            sha256 = "1x9rkyhbp15dwp6dikzpk9lzjnh9cnxac89gzx533681zld906m8";
          }
        ];
        vscodium-with-extensions = pkgs.vscode-with-extensions.override {
          vscode = pkgs.vscodium;
          vscodeExtensions = extensions;
        };
      in
      {

        # nix build
        defaultPackage = buildPackage {
          pname = "bulbb";
          nativeBuildInputs = with pkgs; [ pkg-config ];
          root = ./.;
        };


        # nix develop
        devShell = pkgs.devshell.mkShell {
          name = "bulbb";
          bash.interactive = ''
            alias ..='cd ..'
            alias c='clear'
            alias countfiles='fd -t f | wc -l'
            alias f='fd . | grep '
            alias folders='du -h --max-depth=1'
            alias folderssort='fd . -d 1 -t d -print0 | xargs -0 du -sk | sort -rn'
            alias gt='cd $(fd -H -t d -j $(nproc) | sk )'
            alias h='history | grep '
            alias l='exa --icons'
            alias la='exa --all --icons'
            alias ll='exa --long --header --git --icons'
            alias ls='exa --icons'
            alias lsa='exa --all --icons'
            alias lsal='exa --long --all --header --git --icons'
            alias lsl='exa --long --header --git --icons'
            alias lsla='exa --long --all --header --git --icons'
            alias mem='free -h --si'
            alias sl='exa --icons'
            alias sla='exa --all --icons'
            alias slal='exa --long --all --header --git --icons'
            alias sll='exa --long --header --git --icons'
            alias slla='exa --long --all --header --git --icons'
            alias topcpu='ps -eo pcpu,pid,user,args | sort -k 1 -r | head -10'
            
            eval "$(${pkgs.starship}/bin/starship init bash)"
            bash ${codeSettings}/bin/vscode-settings
          '';

          # Custom scripts. Also easy to use them in CI/CD
          commands = [
            {
              category = "bulbb";
              name = "build";
              help = "Build bulbb";
              command = "cargo build";
            }
            {
              category = "bulbb";
              name = "code";
              help = "Open bulbb in vscodium";
              command = "${vscodium-with-extensions}/bin/codium $PRJ_ROOT";
            }
            {
              category = "bulbb";
              name = "fmt";
              help = "Check formatting for bulbb";
              command = "nixpkgs-fmt \${@} $PRJ_ROOT && cargo fmt";
            }
            {
              category = "bulbb";
              name = "run";
              help = "Run bulbb";
              command = "cargo run";
            }
            {
              category = "bulbb";
              name = "test";
              help = "Run test for bulbb";
              command = "cargo test";
            }
            { 
              category = "bulbb";
              name = "nf";
              help = "Create new file in bulbb src directory";
              command = "${newFile}/bin/newFile";
            }
          ];

          packages = with pkgs;[
            stdenv.cc
            rust-stable
            rust-analyzer
            cargo-whatfeatures
            cargo-release
            cargo-license
            cargo-tarpaulin

            # for shell
            zsh
            neofetch
            nixpkgs-fmt
            starship
            fd
            ripgrep
            exa
          ] ++ [ vscodium-with-extensions ];
          env = [
            { name = "RUST_SRC_PATH"; eval = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}"; }
          ];
        };
      });
}
