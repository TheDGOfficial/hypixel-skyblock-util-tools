# TODO list for the project

# Performance

Do PGO, should be easy as this a CLI application: https://doc.rust-lang.org/rustc/profile-guided-optimization.html

Basically optimize for most common inputs to every selection/tool

Also consider making the default requirement x86_64v2 instead of x86_64.

Maybe also do BOLT on top of PGO. https://github.com/llvm/llvm-project/tree/main/bolt

# Speeding up dev/debug builds

general: use sccache for compiler cache, see https://github.com/mozilla/sccache

# Dependencies

Cargo.toml: reqwest stable doesn't have HTTP/3 support, so we are using a fork, but that fork is based on 1 release
behind than stable

# Making release build smaller (might decrease execution or startup speed plus there are sayings that UPX makes antivirus software flag the executable)

release_build.sh: upx doesn't work because of a gnu dt hash table error, figure it out

# Portability

Build scripts: Either build an actually portable executable that works on (at least) Windows/macOS/Linux or build
executables for Windows & macOS along with the current Linux-only executable.

Cargo.toml: we might want to switch to musl (or cosmopolitian libc) instead of gnu for c library and compile to i686 to
support 32-bit OS's.

# The Actual Code

main.rs: use parallel iterator with rayon to increase performance
main.rs: http2\_prior\_knowledge() gives out errors, fix it and make it http3\_prior\_knowledge() in the future

main.rs: (general) make everything concurrent and parallel

