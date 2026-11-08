Elyhorizon for Linux x86_64

Recommended: download the Elyhorizon AppImage, make it executable, and run it:
  chmod +x Elyhorizon-<version>-linux-x86_64.AppImage
  ./Elyhorizon-<version>-linux-x86_64.AppImage

If FUSE is unavailable, add --appimage-extract-and-run to the launch command.
The AppImage includes both the launcher and its matching Horizon client.

For the tar.gz archive, extract the entire folder and run ./horizon-launcher.
Keep horizon-launcher and horizon-client together. The archive uses your system
libraries; use the AppImage if a required launcher library is missing.

Built on Ubuntu 22.04 (glibc 2.35). Use a compatible desktop distribution,
working graphics drivers, and a browser for Ely.by sign-in. Secret Service is
used when available; the launcher reports credential-store limitations.
The public Desktop application ID is built in; no client secret is needed.
Minecraft game resources are downloaded or prepared separately at runtime.
