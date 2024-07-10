let
  
  pkgs = import <nixpkgs> {
    config.allowUnfree = true;
  };
in pkgs.mkShell {
  packages = [
  rustc
  cargo
  ];
}
