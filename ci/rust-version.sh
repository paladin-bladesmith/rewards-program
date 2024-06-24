#
# This file maintains the rust versions for use by CI.
#
# Obtain the environment variables without any automatic toolchain updating:
#   $ source ci/rust-version.sh
#

if [[ -n $RUST_STABLE_VERSION ]]; then
  stable_version="$RUST_STABLE_VERSION"
else
  base="$(dirname "${BASH_SOURCE[0]}")"
  source "$base/read-cargo-variable.sh"
  stable_version=$(readCargoVariable toolchain channel "$base/../rust-toolchain.toml")
fi

if [[ -n $RUSTFMT_NIGHTLY_VERSION ]]; then
  rustfmt_nightly_version="$RUSTFMT_NIGHTLY_VERSION"
else
  base="$(dirname "${BASH_SOURCE[0]}")"
  source "$base/read-cargo-variable.sh"
  rustfmt_nightly_version="$(readCargoVariable workspace.metadata.scripts.rustfmt.toolchain channel "$base/../Cargo.toml")"
fi

if [[ -n $CLIPPY_NIGHTLY_VERSION ]]; then
  clippy_nightly_version="$CLIPPY_NIGHTLY_VERSION"
else
  base="$(dirname "${BASH_SOURCE[0]}")"
  source "$base/read-cargo-variable.sh"
  clippy_nightly_version="$(readCargoVariable workspace.metadata.scripts.clippy.toolchain channel "$base/../Cargo.toml")"
fi

export rust_stable="$stable_version"
export rustfmt_nightly_version="$rustfmt_nightly_version"
export clippy_nightly_version="$clippy_nightly_version"

[[ -z $1 ]] || (

  rustup_install() {
    declare toolchain=$1
    if ! cargo +"$toolchain" -V > /dev/null; then
      echo "$0: Missing toolchain? Installing...: $toolchain" >&2
      rustup install "$toolchain"
      cargo +"$toolchain" -V
    fi
  }

  set -e
  cd "$(dirname "${BASH_SOURCE[0]}")"
  case $1 in
  stable)
    rustup_install "$rust_stable"
    ;;
  nightly)
    rustup_install "$rust_nightly"
    ;;
  all)
    rustup_install "$rust_stable"
    rustup_install "$rust_nightly"
    ;;
  *)
    echo "$0: Note: ignoring unknown argument: $1" >&2
    ;;
  esac
)