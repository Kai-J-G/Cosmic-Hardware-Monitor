pkgname=Cosmic-Hardware-Monitor
appid='io.github.kaijg.CosmicHardwareMonitor'
pkgver=1.0.0
pkgrel=1
pkgdesc='Native hardware and thermal monitor applet for the COSMIC desktop — CPU, GPU, memory and storage, read straight from the kernel.'
url='https://github.com/Kai-J-G/Cosmic-Hardware-Monitor'
license=("MIT license")
makedepends=('cargo' 'just')
depends=('libxkbcommon' 'pkgconf' 'pciutils')
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
source=()
b2sums=()

prepare() {
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --release
}

check() {
    export RUSTUP_TOOLCHAIN=stable
    cargo check
    cargo test
}

package() {
    just prefix="$pkgdir/usr" install

    # undo the build-dir path the recipe seds into the .desktop
    sed -i "s|Exec=.*/|Exec=/usr/bin/|" \
        "$pkgdir/usr/share/applications/$appid.desktop"
}