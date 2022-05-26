# shaderlab

## Installation
[RustUp](https://www.rust-lang.org/tools/install) will install Rust, there may be a better installer depending on your operating system.

### [Fedora](https://getfedora.org/)
incomplete list
`sudo dnf install alsa-lib-devel`

### [Ubuntu](https://ubuntu.com/)
incomplete list
`sudo apt-get install libasound2-dev libudev-dev`

### [Clear Linux OS](https://clearlinux.org/)

Rust
`sudo swupd bundle-add rust-basic`

ALSA developer library
`sudo swupd bundle-add devpkg-alsa-lib`

libudev
`sudo swupd bundle-add devpkg-libgudev`

### [OpenSUSE]
incomplete list
`sudo zypper install libudev-devel alsa-lib-devel`

## Running

`cargo run`
