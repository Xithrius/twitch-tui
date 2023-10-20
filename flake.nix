{
  description = "Twitch chat in the terminal.";
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
  let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
    manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
  in {
    packages.${system}.default = pkgs.rustPlatform.buildRustPackage {
      pname = manifest.name;
      version = manifest.version;

      buildInputs = [pkgs.openssl.dev];
      nativeBuildInputs = [pkgs.pkg-config];

      src = pkgs.lib.cleanSource ./.;
      cargoLock.lockFile = ./Cargo.lock;
    };

    apps.${system}.default = {
      type = "app";
      program = "${self.packages.${system}.default}/bin/twt";
    };
  };
}
