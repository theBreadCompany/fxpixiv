# fxpixiv

The pixiv embed fixer

## DISCLAIMER

This project is still in development, please be patient until this project is actually usable.

## build

Simply use `cargo build` and/or `cargo run`.

**Mind that you need a refresh token from pixiv to get API access.**


## nix build

As this mainly runs on my NixOS server for now, so build and service files are included. 
Run `nix-shell -E 'with import <nixpkgs> {}; callPackage ./default.nix {}'` to enter the development shell.

## nix options

There is some hope that this service will make its way to the NixOS options, but I need to disappoint you for now.
You can still just clone this project and manually import the service file of course, make sure to keep the `default.nix` reachable tho.