{
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  extraBuildInputs ? "",
}:

pimalaya.mkShell {
  extraBuildInputs = "nixd,nixfmt-rfc-style,git-cliff" + extraBuildInputs;
}
