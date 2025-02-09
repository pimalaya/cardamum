# TODO: move this to nixpkgs
# This file aims to be an up-to-date replacement on master for the nixpkgs derivation.

{ lib
, pkg-config
, buildPackages
, rustPlatform
, fetchFromGitHub
, stdenv
, apple-sdk
, installShellFiles
, installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform
, installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform
, withNoDefaultFeatures ? false
, withFeatures ? [ ]
,
}:

let
  version = "0.1.0";
  hash = "";
  cargoHash = "";
in rustPlatform.buildRustPackage {
  inherit cargoHash version;

  pname = "cardamum";

  src = fetchFromGitHub {
    inherit hash;
    owner = "pimalaya";
    repo = "cardamum";
    rev = "v${version}";
  };

  buildNoDefaultFeatures = withNoDefaultFeatures;
  buildFeatures = withFeatures;

  nativeBuildInputs = [
    pkg-config
  ] ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs = lib.optional stdenv.hostPlatform.isDarwin apple-sdk;

  # unit tests only
  cargoTestFlags = [ "--lib" ];
  doCheck = false;
  auditable = false;

  postInstall = let emulator = stdenv.hostPlatform.emulator buildPackages; in
    ''
      mkdir -p $out/share/{completions,man}
      ${emulator} "$out"/bin/cardamum man "$out"/share/man
      ${emulator} "$out"/bin/cardamum completion bash > "$out"/share/completions/cardamum.bash
      ${emulator} "$out"/bin/cardamum completion elvish > "$out"/share/completions/cardamum.elvish
      ${emulator} "$out"/bin/cardamum completion fish > "$out"/share/completions/cardamum.fish
      ${emulator} "$out"/bin/cardamum completion powershell > "$out"/share/completions/cardamum.powershell
      ${emulator} "$out"/bin/cardamum completion zsh > "$out"/share/completions/cardamum.zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion "$out"/share/completions/cardamum.{bash,fish,zsh}
    '';

  meta = with lib; {
    description = "CLI to manage contacts";
    mainProgram = "cardamum";
    homepage = "https://github.com/pimalaya/cardamum";
    changelog = "https://github.com/pimalaya/cardamum/blob/v${version}/CHANGELOG.md";
    license = licenses.mit;
    maintainers = with maintainers; [ soywod ];
  };
}
