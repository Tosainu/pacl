# pacl

![](https://github.com/Tosainu/pacl/workflows/CI/badge.svg)

Simple `git clone` wrapper.

## Example

```
$ pacl rust-lang/rust -- --depth 50
➔ git clone \
    https://github.com/rust-lang/rust \
    ~/.pacl/github.com/rust-lang/rust \
    --depth 50

$ https://gitlab.freedesktop.org/xorg/app/xeyes
➔ git clone \
    https://gitlab.freedesktop.org/xorg/app/xeyes \
    ~/.pacl/gitlab.freedesktop.org/xorg/app/xeyes
```
