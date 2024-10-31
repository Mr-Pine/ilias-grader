{
  inputs = {
    naersk.url = "github:nix-community/naersk/master";
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    naersk.inputs.nixpkgs.follows = "nixpkgs";
  };
  outputs =
    {
      self,
      nixpkgs,
      naersk,
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;
    in
    {
      packages = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
          naersk-lib = pkgs.callPackage naersk { };
        in
        {
          default = naersk-lib.buildPackage {
            src = ./.;
            nativeBuildInputs = with pkgs; [
              openssl
              pkg-config
              dbus
            ];
          };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs { inherit system; };
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustc
              rustfmt
              rustPackages.clippy
            ];
            nativeBuildInputs = with pkgs; [
              openssl
              pkg-config
              dbus
            ];
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        }
      );

      formatter = forAllSystems (system: (import nixpkgs { inherit system; }).nixfmt-rfc-style);
    };
}
