{
  buildFeatures ? [ ],
  buildNoDefaultFeatures ? false,
  buildPackages,
  fetchFromGitHub,
  installManPages ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellCompletions ? stdenv.buildPlatform.canExecute stdenv.hostPlatform,
  installShellFiles,
  lib,
  openssl,
  pkg-config,
  rustPlatform,
  stdenv,
}:

let
  version = "0.1.0";
  emul = stdenv.hostPlatform.emulator buildPackages;
  exe = stdenv.hostPlatform.extensions.executable;

in
rustPlatform.buildRustPackage {
  inherit version buildNoDefaultFeatures buildFeatures;

  pname = "cardamum";
  cargoHash = "";

  src = fetchFromGitHub {
    hash = "";
    owner = "pimalaya";
    repo = "cardamum";
    rev = "v${version}";
  };

  env.OPENSSL_NO_VENDOR = true;

  nativeBuildInputs = [
    pkg-config
    installShellFiles
  ];

  buildInputs = lib.optional (builtins.elem "native-tls" buildFeatures) openssl;

  # most of the tests are lib side
  doCheck = false;

  postInstall =
    lib.optionalString (lib.hasInfix "wine" emul) ''
      export WINEPREFIX="''${WINEPREFIX:-$(mktemp -d)}"
      mkdir -p $WINEPREFIX
    ''
    + ''
      mkdir -p $out/share/{applications,completions,man}
      ${emul} "$out"/bin/cardamum${exe} manuals "$out"/share/man
      ${emul} "$out"/bin/cardamum${exe} completions -d "$out"/share/completions bash elvish fish powershell zsh
    ''
    + lib.optionalString installManPages ''
      installManPage "$out"/share/man/*
    ''
    + lib.optionalString installShellCompletions ''
      installShellCompletion --cmd cardamum \
        --bash "$out"/share/completions/cardamum.bash \
        --fish "$out"/share/completions/cardamum.fish \
        --zsh "$out"/share/completions/_cardamum
    '';

  meta = {
    description = "CLI to manage contacts";
    mainProgram = "cardamum";
    homepage = "https://github.com/pimalaya/cardamum";
    changelog = "https://github.com/pimalaya/cardamum/blob/master/CHANGELOG.md";
    license = with lib.licenses; [
      mit
      asl20
    ];
    maintainers = with lib.maintainers; [ soywod ];
  };
}
