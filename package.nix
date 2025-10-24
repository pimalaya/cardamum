# TODO: move this to nixpkgs
# This file aims to be a replacement for the nixpkgs derivation.

{
  lib,
  pkg-config,
  rustPlatform,
  fetchFromGitHub,
  stdenv,
  apple-sdk,
  installShellFiles,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  withNoDefaultFeatures ? false,
  withFeatures ? [ ],
}:

let
  version = "0.1.0";
  hash = "";
  cargoHash = "";
in

rustPlatform.buildRustPackage rec {
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
  ]
  ++ lib.optional (installManPages || installShellCompletions) installShellFiles;

  buildInputs = lib.optional stdenv.hostPlatform.isDarwin apple-sdk;

  # unit tests only
  cargoTestFlags = [ "--lib" ];
  doCheck = false;
  auditable = false;

  postInstall = ''
    mkdir -p $out/share/{completions,man}
  ''
  + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/cardamum man "$out"/share/man
  ''
  + lib.optionalString installManPages ''
    installManPage "$out"/share/man/*
  ''
  + lib.optionalString (stdenv.buildPlatform.canExecute stdenv.hostPlatform) ''
    "$out"/bin/cardamum completion bash > "$out"/share/completions/cardamum.bash
    "$out"/bin/cardamum completion elvish > "$out"/share/completions/cardamum.elvish
    "$out"/bin/cardamum completion fish > "$out"/share/completions/cardamum.fish
    "$out"/bin/cardamum completion powershell > "$out"/share/completions/cardamum.powershell
    "$out"/bin/cardamum completion zsh > "$out"/share/completions/cardamum.zsh
  ''
  + lib.optionalString installShellCompletions ''
    installShellCompletion "$out"/share/completions/cardamum.{bash,fish,zsh}
  '';

  meta = {
    description = "CLI to manage contacts";
    mainProgram = "cardamum";
    homepage = "https://github.com/pimalaya/cardamum";
    changelog = "https://github.com/pimalaya/cardamum/blob/v${version}/CHANGELOG.md";
    license = lib.licenses.agpl3Only;
    maintainers = with lib.maintainers; [
      soywod
    ];
  };
}
