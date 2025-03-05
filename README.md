# Building on NixOS

Building on NixOS is a bit more involved, as it does not follow a traditional Unix-based package layout and therefore some adjustments have to be made.

Next, export the following variables

```
export OPENSSL_DIR=/nix/store/{sha256}-openssl-{version}-dev/
export OPENSSL_LIB_DIR=/nix/store/{sha256}-openssl-{version}/lib
```
