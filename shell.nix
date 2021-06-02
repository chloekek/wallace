{ nixpkgs ? import nix/nixpkgs }:

nixpkgs.mkShell {
    nativeBuildInputs = [
        nixpkgs.libsodium
        nixpkgs.wallace.pinned.cargo
    ];
}
