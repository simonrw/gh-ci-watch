{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell rec {
  packages = [
    rustup
    clang
    bacon
  ] ++ lib.optionals stdenv.isDarwin (with darwin.apple_sdk.frameworks; [
    libiconv
    Cocoa
  ]) ++ lib.optionals stdenv.isLinux [
    mold
  ];

  shellHook = ''
    export RUST_BUILD_BASE="$HOME/.cache/rust-builds"
    WORKSPACE_ROOT=$(cargo metadata --no-deps --offline 2>/dev/null | jq -r ".workspace_root")
    PACKAGE_BASENAME=$(basename $WORKSPACE_ROOT)

    # Run cargo with target set to $RUST_BUILD_BASE/$PACKAGE_BASENAME
    export CARGO_TARGET_DIR="$RUST_BUILD_BASE/$PACKAGE_BASENAME"
  '';

  env = {
    RUST_SRC_PATH = "${rustPlatform.rustLibSrc}";
    LD_LIBRARY_PATH = lib.makeLibraryPath packages;
  };
}
