# Maintainer: Ivan Chinenov <ichinenov@hjvt.dev>

pkgname=niri-wsr
pkgver=25.8.0
pkgrel=1
pkgdesc="Automatic workspace renamer for niri inspired by i3-wsr"
url="https://github.com/JohnDowson/$pkgname"
license=('MIT')
makedepends=('cargo')
depends=()
arch=('i686' 'x86_64' 'armv6h' 'armv7h')
source=("$pkgname-$pkgver.tar.gz::$url/releases/download/v$pkgver/code.tar.gz")
sha512sums=({{SHA512}})

prepare() {
    cargo fetch --locked --target "$(rustc -vV | sed -n 's/host: //p')"
}

build() {
    export CARGO_TARGET_DIR=target
    cargo build --frozen --release --all-features
}

check() {
    cargo test --frozen --all-features
}

package() {
    install -Dm0755 -t "$pkgdir/usr/bin/" "target/release/$pkgname"
    install -Dm0755 -t "$pkgdir/usr/lib/systemd/user/" "$pkgname.service"
}