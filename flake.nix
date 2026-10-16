{
  description = "ardemu";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      flake-utils,
    }:
    let
      inherit (nixpkgs) lib;

      cargoToml = lib.importTOML ./ardemu_gui/Cargo.toml;
      pname = cargoToml.package.name;

      rustBinFor = pkgs: (pkgs.extend rust-overlay.overlays.default).rust-bin;
      toolchainFor = pkgs: (rustBinFor pkgs).selectLatestNightlyWith (toolchain: toolchain.minimal);
      devToolchainFor =
        pkgs:
        (rustBinFor pkgs).selectLatestNightlyWith (
          toolchain:
          toolchain.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          }
        );
      runtimeLibsFor =
        pkgs: with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader

          # X11 fallback
          libx11
          libxcursor
          libxrandr
          libxi
        ];
      depsFor = pkgs: {
        nativeBuildInputs = with pkgs; [ pkg-config ];
        buildInputs =
          runtimeLibsFor pkgs
          ++ (with pkgs; [
            # for iced_comet debugger
            openssl.dev
          ]);
      };
    in
    {
      overlays.default = final: _prev: {
        ${pname} =
          let
            toolchain = toolchainFor final;
            rustPlatform = final.makeRustPlatform {
              cargo = toolchain;
              rustc = toolchain;
            };
          in
          rustPlatform.buildRustPackage (
            {
              inherit pname;
              inherit (cargoToml.package) version;

              src = lib.fileset.toSource {
                root = ./.;
                fileset = lib.fileset.unions [
                  ./Cargo.toml
                  ./Cargo.lock
                  ./src
                  ./ardemu_core
                  ./ardemu_gui
                  ./ardemu_macros
                  ./sample_programs
                  ./tests
                ];
              };
              cargoLock.lockFile = ./Cargo.lock;
              strictDeps = true;
              postFixup = lib.optionalString final.stdenv.hostPlatform.isLinux ''
                patchelf --add-rpath ${lib.makeLibraryPath (runtimeLibsFor final)} $out/bin/${pname}
              '';
              meta = {
                description = cargoToml.package.description or pname;
                license = lib.licenses.mit;
                mainProgram = pname;
              };
            }
            // depsFor final
          );
      };
    }
    // flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };
      in
      {
        packages = {
          default = pkgs.${pname};
          ${pname} = pkgs.${pname};
        };

        apps.default = flake-utils.lib.mkApp { drv = pkgs.${pname}; };

        devShells.default = pkgs.mkShell {
          inherit (depsFor pkgs) nativeBuildInputs buildInputs;
          packages = [ (devToolchainFor pkgs) ];
          env.RUST_BACKTRACE = "1";
          shellHook = ''
            export LD_LIBRARY_PATH="${lib.makeLibraryPath (runtimeLibsFor pkgs)}''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
          '';
        };

        formatter = pkgs.nixfmt-tree;
      }
    );
}
