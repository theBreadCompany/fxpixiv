{ pkgs, lib, rustPlatform }:

rustPlatform.buildRustPackage rec {
  pname = "fxpixiv";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  checkFlags = [
    "--skip=client_tests::login --skip=client_tests::illust_details"
  ];

  cargoSha256 = lib.fakeSha256; # Replace with the actual hash

  nativeBuildInputs = [ pkgs.pkg-config ];
  buildInputs = [ pkgs.openssl ];

  meta = with lib; {
    description = "a pixiv embed helper";
    license = licenses.mit;
    maintainers = [ ];
  };
}
