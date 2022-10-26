# TODO list for the project

# Lint/Analyzing

run.sh: figure out how to setup MIRAI for even more code smells

# Speeding up dev/debug builds

general: use sccache for compiler cache, see https://github.com/mozilla/sccache

Cargo.toml: trying to use mold linker causes unknown argument error, figure it out

# Dependencies

Cargo.toml: reqwest stable doesn't have HTTP/3 support so we are using a fork, but that fork is based on 1 release behind than stable

# Making release build smaller (might decrease execution or startup speed plus there are sayings that UPX makes anti-virus software flag the executable)

release_build.sh: upx doesn't work because of a gnu dt hash table error, figure it out

# Portability

Build scripts: Either build a actually portable executable that works on (at least) Windows/macOS/Linux or build executables for Windows & macOS along with the current Linux-only executable.

Cargo.toml: we might want to switch to musl instead of gnu for c library

# The Actual Code

main.rs: use parallel iterator with rayon to increase performance
main.rs: http2\_prior\_knowledge() gives out errors, fix it and make it http3\_prior\_knowledge() in the future

main.rs: (general) make everything concurrent and parallel

