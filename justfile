version := `sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1`
target := `rustc -vV | sed -n 's/^host: //p'`

default:
    @just --list

fmt:
    cargo fmt --check

check:
    cargo check --locked

test:
    cargo test --locked

build:
    cargo build --release --locked

run:
    cargo run --locked

verify: fmt check test

fedora-vendor:
    mkdir -p target/rpm
    cargo vendor --locked target/rpm/vendor > target/rpm/vendor-config.toml

fedora-source: fedora-vendor
    mkdir -p target/rpm/SOURCES target/rpm/source/iguard-{{version}}/.cargo
    cp Cargo.toml Cargo.lock LICENSE README.md target/rpm/source/iguard-{{version}}/
    cp -a src packaging target/rpm/source/iguard-{{version}}/
    cp -a target/rpm/vendor target/rpm/source/iguard-{{version}}/vendor
    printf '[source.crates-io]\nreplace-with = "vendored-sources"\n\n[source.vendored-sources]\ndirectory = "vendor"\n' > target/rpm/source/iguard-{{version}}/.cargo/config.toml
    tar -czf target/rpm/SOURCES/iguard-{{version}}.tar.gz -C target/rpm/source iguard-{{version}}

fedora-rpm: fedora-source
    mkdir -p target/rpm/BUILD target/rpm/BUILDROOT target/rpm/RPMS target/rpm/SPECS target/rpm/SRPMS
    cp packaging/fedora/iguard.spec target/rpm/SPECS/iguard.spec
    command -v rpmbuild >/dev/null || { echo 'rpmbuild is required (run this recipe on Fedora with RPM build tools installed)' >&2; exit 1; }
    rpmbuild -ba target/rpm/SPECS/iguard.spec --define "_topdir $PWD/target/rpm"

release-assets:
    cargo build --release --locked
    mkdir -p target/release-assets
    tar -czf target/release-assets/iguard-v{{version}}-{{target}}.tar.gz -C target/release iguard
    cd target/release-assets && sha256sum iguard-v{{version}}-{{target}}.tar.gz > SHA256SUMS
