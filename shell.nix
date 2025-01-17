let
	pkgs = import (fetchTarball "https://github.com/NixOS/nixpkgs/tarball/nixos-24.11") {};
in
pkgs.mkShell {
	packages = [
		(pkgs.python3.withPackages (python-pkgs: with python-pkgs; [
			requests
			beautifulsoup4
			prettytable
		]))
		pkgs.sqlite
	];
}
