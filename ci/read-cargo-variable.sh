# source this file

readCargoVariable() {
  declare section="$1"
  declare variable="$2"
  declare Cargo_toml="$3"

  SECTION=$(sed -n "/^\[$section\]/,/^\[/ {
    /^\[$section\]/! {
      /^\[/ q
      p
    }
  }" $Cargo_toml)

  while read -r name equals value _; do
    if [[ $name = "$variable" && $equals = = ]]; then
      echo "${value//\"/}"
      return
    fi
  done <<< "$SECTION"
  echo "Unable to locate $section.$variable in $Cargo_toml" 1>&2
}