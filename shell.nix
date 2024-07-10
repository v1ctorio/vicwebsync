let
  
  pkgs = import <nixpkgs> {
    config.allowUnfree = true;
  };
in pkgs.mkShell {
  packages = with pkgs; [
  rustc
  cargo
  openssl
  pkg-config
  ];
}
