# Maintainer: Kai Gurney <kai.gurney@epita.fr>
# Contributor: prushton2

pkgname=cosmic-ext-hardware-monitor
_appid=io.github.kai_j_g.CosmicHardwareMonitor
_repo=CosmicHardwareMonitor
pkgver=1.1.1
pkgrel=1
pkgdesc='Native hardware and thermal monitor applet for the COSMIC desktop'
arch=('x86_64' 'aarch64')
url="https://github.com/Kai-J-G/$_repo"
license=('MIT')
depends=('libxkbcommon')
makedepends=('cargo')
optdepends=(
    'pciutils: read a GPU name that the driver does not expose'
    'nvidia-utils: NVIDIA telemetry via nvidia-smi'
)
source=("$pkgname-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz")
sha256sums=('3692e2cb416e6b09058fd4329a3c524824314b53886777fbd75a4a961bde8067')

prepare() {
    cd "$_repo-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    cd "$_repo-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release
}

check() {
    cd "$_repo-$pkgver"
    export RUSTUP_TOOLCHAIN=stable
    # Reads /sys and /proc, both of which makepkg's build chroot provides.
    cargo test --frozen
}

package() {
    cd "$_repo-$pkgver"

    install -Dm0755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"

    # The desktop entry ships with a bare Exec=, which resolves on PATH for a
    # system install, so unlike `just install` there is nothing to rewrite.
    install -Dm0644 "data/$_appid.desktop" "$pkgdir/usr/share/applications/$_appid.desktop"
    install -Dm0644 "data/$_appid.metainfo.xml" "$pkgdir/usr/share/metainfo/$_appid.metainfo.xml"

    local icons="$pkgdir/usr/share/icons/hicolor/scalable/apps"
    install -Dm0644 "data/icons/$_appid.svg" "$icons/$_appid.svg"
    install -Dm0644 "data/icons/$_appid-symbolic.svg" "$icons/$_appid-symbolic.svg"

    install -Dm0644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
