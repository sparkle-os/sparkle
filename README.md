![development: sporadic](https://img.shields.io/badge/development-sporadic-yellowgreen.svg) [![dependency status](https://deps.rs/repo/github/sparkle-os/sparkle/status.svg)](https://deps.rs/repo/github/sparkle-os/sparkle)

# ✨sparkle✨

![a screenshot of the Sparkle boot screen](docs/sparkle.png)

## building
system prereqs/deps:
* qemu
* grub (`grub-mkrescue` is used to generate the bootable `.iso`)

```
$ cargo install cargo-xbuild
$ make
```

to run in an emulator:
```
$ make run
```
