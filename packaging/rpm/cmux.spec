Name:           cmux
Version:        %{_cmux_version}
Release:        1%{?dist}
Summary:        GPU-accelerated terminal multiplexer
License:        Proprietary
URL:            https://cmux.dev

# Pre-built binary package -- no Source0, no %build
AutoReqProv:    no

Requires:       gtk4
Requires:       fontconfig
Requires:       freetype
Requires:       oniguruma
Requires:       mesa-libGL
Requires:       mesa-libEGL
Requires:       harfbuzz
Requires:       glib2
Requires:       cairo
Requires:       cairo-gobject
Requires:       pango
Requires:       gdk-pixbuf2
Requires:       libepoxy
Requires:       libxkbcommon
Requires:       graphene

%description
cmux is a GPU-accelerated terminal with tabs, splits, workspaces,
and socket CLI control -- powered by Ghostty.

%install
install -Dm0755 %{_sourcedir}/cmux-app %{buildroot}%{_bindir}/cmux-app
install -Dm0755 %{_sourcedir}/cmux %{buildroot}%{_bindir}/cmux
install -Dm0755 %{_sourcedir}/cmuxd-remote %{buildroot}%{_libdir}/cmux/cmuxd-remote

install -Dm0644 %{_sourcedir}/com.cmux_lx.terminal.desktop %{buildroot}%{_datadir}/applications/com.cmux_lx.terminal.desktop
install -Dm0644 %{_sourcedir}/com.cmux_lx.terminal.metainfo.xml %{buildroot}%{_datadir}/metainfo/com.cmux_lx.terminal.metainfo.xml

install -Dm0644 %{_sourcedir}/icons/48x48.png %{buildroot}%{_datadir}/icons/hicolor/48x48/apps/com.cmux_lx.terminal.png
install -Dm0644 %{_sourcedir}/icons/128x128.png %{buildroot}%{_datadir}/icons/hicolor/128x128/apps/com.cmux_lx.terminal.png
install -Dm0644 %{_sourcedir}/icons/256x256.png %{buildroot}%{_datadir}/icons/hicolor/256x256/apps/com.cmux_lx.terminal.png

install -Dm0644 %{_sourcedir}/cmux.bash %{buildroot}%{_datadir}/bash-completion/completions/cmux
install -Dm0644 %{_sourcedir}/_cmux %{buildroot}%{_datadir}/zsh/vendor-completions/_cmux
install -Dm0644 %{_sourcedir}/cmux.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/cmux.fish

install -Dm0644 %{_sourcedir}/cmux.1.gz %{buildroot}%{_mandir}/man1/cmux.1.gz

%files
%{_bindir}/cmux-app
%{_bindir}/cmux
%{_libdir}/cmux/cmuxd-remote
%{_datadir}/applications/com.cmux_lx.terminal.desktop
%{_datadir}/metainfo/com.cmux_lx.terminal.metainfo.xml
%{_datadir}/icons/hicolor/48x48/apps/com.cmux_lx.terminal.png
%{_datadir}/icons/hicolor/128x128/apps/com.cmux_lx.terminal.png
%{_datadir}/icons/hicolor/256x256/apps/com.cmux_lx.terminal.png
%{_datadir}/bash-completion/completions/cmux
%{_datadir}/zsh/vendor-completions/_cmux
%{_datadir}/fish/vendor_completions.d/cmux.fish
%{_mandir}/man1/cmux.1.gz
