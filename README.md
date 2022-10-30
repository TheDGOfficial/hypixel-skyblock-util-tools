# hypixel-skyblock-util-tools

Command-line utility tools for Hypixel SkyBlock, written in Rust.

# Note

This project is in development and there are no officially released binaries yet.

You can grab a nightly binary that works on 64-bit Linux from the GitHub actions. There is no Windows or macOS support
at the moment.

You can also compile it yourself, this will make use of the -march=native -mtune=native arguments instead of the
-march=x86_64 -mtune=generic we use on the GitHub Actions for portability, and will produce a faster binary. Although,
this a simple CLI application, so there's not much need to do that.

