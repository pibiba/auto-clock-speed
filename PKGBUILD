# Maintainer: jakeroggenbuck <jakeroggenbuck2@gmail.com
# Maintainer: cameron <cameron.bannasch@gmail>
# Maintainer: shuzhengz <treez.zhang@gmail.com>
#
# This PKGBUILD was generated by `cargo aur`: https://crates.io/crates/cargo-aur

pkgname=autoclockspeed-bin
pkgver=0.1.9
pkgrel=1
pkgdesc="A utility to check stats about your CPU, and auto regulate clock speeds to help with either performance or battery life."
url="https://github.com/JakeRoggenbuck/auto-clock-speed"
license=("MIT")
arch=("x86_64")
provides=("autoclockspeed")
conflicts=("autoclockspeed")
source=("https://github.com/JakeRoggenbuck/auto-clock-speed/releases/download/v$pkgver/autoclockspeed-$pkgver-x86_64.tar.gz")
sha256sums=("a85616cd8861e76c11791584879defa70b03c88f86a33ce86c348c7251227a1c")

package() {
    install -Dm755 acs -t "$pkgdir/usr/bin"
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
}
