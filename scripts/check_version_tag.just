# Check that the version specified in Cargo.toml matches the latest git tag version
check-version:
    #!/usr/bin/env bash
    set -euo pipefail
    git fetch --tags
    cargo_ver=$( just _get_cargo_toml_version )
    git_tag_ver=$( just _latest_git_tag )
    if [[ ${cargo_ver} != ${git_tag_ver} ]]; then
        {{PRINT}} red "Mismatch between Cargo.toml and git tag version ${cargo_ver} != ${git_tag_ver}"
        exit 1
    fi

# Get the version as specified in Cargo.toml
_get_cargo_toml_version:
    grep --perl-regexp --only-matching "(?<=version = \")[0-9.]*" Cargo.toml

# Get the semver described by the latest tag
_latest_git_tag:
    git describe --tags --abbrev=0 | grep --perl-regexp --only-matching "(?<=v).*"
