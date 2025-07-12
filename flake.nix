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
        ];
      };
    };
}
