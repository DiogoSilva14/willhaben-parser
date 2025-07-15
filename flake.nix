{
  description = "Rust flake";
  inputs = { nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05"; };
  outputs = { self, nixpkgs, ... }@inputs:
    let
     system = "x86_64-linux";
     pkgs = nixpkgs.legacyPackages.${system};
    in
    {
      devShells.${system}.default = pkgs.mkShell
      {
        packages = with pkgs; [
          rustc 
          cargo 
          pkg-config
          openssl
          sqlite
        ];
      };
      shellHook = ''
        export OPENSSL_STATIC=1
        export OPENSSL_LIB_DIR="${pkgs.openssl}/lib"
        export OPENSSL_INCLUDE_DIR="${pkgs.openssl}/include"
      '';
    };
}
