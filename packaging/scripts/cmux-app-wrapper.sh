#!/bin/sh
# cmux-app launcher — auto-detects display backend for GTK4 GL compatibility.
#
# GTK4 may choose Wayland/EGL even in X11 sessions if Wayland libraries are
# present, causing GL context creation failures on NVIDIA proprietary drivers.
# This wrapper forces X11/GLX when running under an X11 session.

if [ -z "$GDK_BACKEND" ]; then
    case "${XDG_SESSION_TYPE}" in
        x11)
            export GDK_BACKEND=x11
            ;;
        wayland)
            # Check for NVIDIA proprietary driver — EGL often fails
            if command -v nvidia-smi >/dev/null 2>&1; then
                export GDK_BACKEND=x11
            fi
            ;;
    esac
fi

exec /usr/bin/cmux-app.bin "$@"
