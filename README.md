# Building on NixOS

Building on NixOS is a bit more involved, as it does not follow a traditional Unix-based package layout and therefore some adjustments have to be made.

First, make sure to install `cargo`, `rustc` and `clang`. Next specify the linker as `clang`:

```
>>> .cargo/config.toml

[target.x86_64-unknown-linux-gnu]
linker = "clang"
```

Next, export the following variables

```

```