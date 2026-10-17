# Packman

A tool to manage installing and uninstalling packages declaratively on Arch Linux.

Packages to be installed or uninstalled are displayed in lexicographical order before changes are made.

The config directory defaults to ~/.config/packman, but alternative locations can be passed.

Package lists in the config directory should use the .packman file extension, and should consist of each package on a different line.

Comments are supported in package lists using the '#' character at the start of a line.

## Flags

  - To update system, use the -u or --upgrade flag

  - To forbid uninstalling packages, use the -f or --forbid-uninstall flag

  - To list packages that should be uninstalled (useful when paired with --forbid-uninstall), use the -l or --list-uninstall flag

  - To pass an alternative config directory, use the -d or --config-directory flag

