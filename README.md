# shaderlab
shaderlab is a scene editor or something. It uses [EGUI](https://github.com/emilk/egui) and [WGPU](https://github.com/gfx-rs/wgpu)

## Installation
[RustUp](https://www.rust-lang.org/tools/install) will install Rust, there may be a better installer depending on your operating system.

### [Fedora](https://getfedora.org/)
incomplete list
`sudo dnf install alsa-lib-devel`

### [Ubuntu](https://ubuntu.com/)
incomplete list
`sudo apt-get install libasound2-dev libudev-dev`

### [Clear Linux OS](https://clearlinux.org/)

Rust, ALSA developer library, libudev

```bash
sudo swupd bundle-add devpkg-alsa-lib
sudo swupd bundle-add devpkg-libgudev
```

### [OpenSUSE](https://www.opensuse.org/)
incomplete list
`sudo zypper install libudev-devel alsa-lib-devel`

## Running

`cargo run`
