{ pkgs ? import <nixpkgs> {}
}:

pkgs.mkShell {
    buildInputs = with pkgs; [
        cargo
        clippy
        openssl.dev
        pkg-config 
        rust-analyzer
        rustc
        rustfmt
    ];
}
