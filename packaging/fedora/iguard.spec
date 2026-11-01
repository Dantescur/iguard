Name:           iguard
Version:        0.1.0
Release:        1%{?dist}
Summary:        Graphical Polkit authentication agent
License:        MIT AND Apache-2.0 AND BSD-2-Clause AND BSD-3-Clause AND BSL-1.0 AND CC0-1.0 AND GPL-3.0-or-later AND ISC AND (LGPL-2.1-or-later WITH GCC-exception-2.0) AND Unicode-3.0 AND Unlicense AND Zlib
Source0:        %{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  desktop-file-utils
BuildRequires:  pkgconfig(libsystemd)
BuildRequires:  pkgconfig(xkbcommon)

%description
iguard is a graphical Polkit authentication agent for Linux desktop sessions.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --offline --locked

%install
install -Dpm0755 target/release/iguard %{buildroot}%{_bindir}/iguard
install -Dpm0644 packaging/fedora/iguard.desktop \
    %{buildroot}%{_sysconfdir}/xdg/autostart/iguard.desktop
install -d %{buildroot}%{_datadir}/licenses/%{name}/third-party
for crate in vendor/*; do
    for license in "$crate"/LICENSE* "$crate"/COPYING* "$crate"/NOTICE*; do
        test -f "$license" || continue
        crate_name=$(basename "$crate")
        license_name=$(basename "$license")
        install -Dpm0644 "$license" \
            "%{buildroot}%{_datadir}/licenses/%{name}/third-party/$crate_name/$license_name"
    done
done

%check
cargo test --offline --locked
desktop-file-validate packaging/fedora/iguard.desktop

%files
%license LICENSE
%license %{_datadir}/licenses/%{name}/third-party
%{_bindir}/iguard
%{_sysconfdir}/xdg/autostart/iguard.desktop

%changelog
* Sat Oct 31 2026 iguard contributors - 0.1.0-1
- Initial Fedora package
