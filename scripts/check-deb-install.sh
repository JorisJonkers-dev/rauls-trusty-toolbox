#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Installs the built .deb in a clean Debian and a clean Ubuntu container, as a user does with `apt install ./raul.deb`
# (issue #506): apt must find every dependency the package declares, the package must be named `raul` and have a
# Maintainer, every library the program opens by name must then be on the system, `raul --version` must run, and the
# window must open and close (RAUL_GUI_SMOKE) under Xvfb.
# Usage: scripts/check-deb-install.sh DIR VERSION      DIR holds raul-VERSION-linux-x64.deb
#        IMAGES="debian:stable-slim" scripts/check-deb-install.sh DIR VERSION    other images
set -euo pipefail

if [ "${1:-}" = "--inside" ]; then
  # Runs inside the container: /deb is the folder with the package.
  version="$2"
  export DEBIAN_FRONTEND=noninteractive
  apt-get update -qq
  # Nothing but the package: its Depends must be enough to start the program.
  apt-get install -y -qq --no-install-recommends /deb/raul-"$version"-linux-x64.deb >/dev/null
  info="$(dpkg -s raul 2>&1)"; echo "$info"
  grep -qx 'Package: raul' <<<"$info" || { echo "::error::the package is not named raul"; exit 1; }
  grep -q '^Maintainer: .' <<<"$info" || { echo "::error::the package has no Maintainer"; exit 1; }
  grep -q '^Depends: .' <<<"$info" || { echo "::error::the package declares no dependencies"; exit 1; }
  if grep -qi 'warning' <<<"$info"; then echo "::error::dpkg warns about the package"; exit 1; fi
  # The libraries raul opens by name (the window toolkit, graphics, audio; found with `grep -a` on the binary) must be
  # on the system after that install, and the binary must still name each of them, so that this list cannot go stale.
  # The engine's optional shader compiler (libvkd3d-shader) is the one library a package cannot require everywhere.
  absent=""
  for n in libX11.so.6 libX11-xcb.so.1 libxcb.so.1 libXcursor.so.1 libXi.so.6 libXrender.so.1 \
    libxkbcommon.so.0 libxkbcommon-x11.so.0 libwayland-client.so.0 libwayland-egl.so.1 libEGL.so.1 libGL.so.1 \
    libvulkan.so.1 libpulse.so.0; do
    grep -qaF "$n" /usr/bin/raul || echo "::warning::raul no longer names $n: take it off the list in $0"
    ldconfig -p | grep -qF "$n " || absent="$absent $n"
  done
  if [ -n "$absent" ]; then echo "::error::libraries raul opens but its dependencies do not install:$absent"; exit 1; fi
  ldd /usr/bin/raul | grep -q "=> not found" && { echo "::error::a linked library is missing"; ldd /usr/bin/raul; exit 1; }
  out="$(raul --version)"; echo "$out"
  head -1 <<<"$out" | grep -qx "raul $version"
  # The window needs a display; Xvfb is added only now so that it cannot hide a missing dependency above.
  apt-get install -y -qq --no-install-recommends xvfb xauth >/dev/null
  out="$(RAUL_GUI_SMOKE=1 LIBGL_ALWAYS_SOFTWARE=1 timeout 120 xvfb-run -a raul)"; echo "$out"
  [ "$out" = "raul: the window opened and closed" ]
  exit 0
fi

dir="$(realpath "${1:?folder with the .deb}")"
version="${2:?version}"
for image in ${IMAGES:-debian:stable-slim ubuntu:22.04}; do
  echo "::group::.deb on $image"
  docker run --rm -v "$dir:/deb:ro" -v "$(realpath "$0"):/check.sh:ro" "$image" bash /check.sh --inside "$version" \
    || { echo "::endgroup::"; echo "::error::the .deb does not install and start on $image"; exit 1; }
  echo "::endgroup::"
done
