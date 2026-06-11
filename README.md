# radar-server
server for the radar viewer controler senior project
## Documentation
Documentation can be found at:
https://voidandcaffeine.github.io/radar-server/radar_server/

## Dependancies
install rustup
install rust via rustup

run with cargo on a system with glibc:
`cargo run --`

Or build with cross for systems without glibc:
`cargo install cross`
### Building for x86\_64-unknown-linux
`cross build --release`
### Building for aarch64-unknown-linux
`cross build --target aarch64-unknown-linux-musl --release`
