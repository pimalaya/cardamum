{
  pimalaya ? import (fetchTarball "https://github.com/pimalaya/nix/archive/master.tar.gz"),
  ...
}:

pimalaya.mkShell {
  extraBuildInputs = "dbus,openssl";
}
