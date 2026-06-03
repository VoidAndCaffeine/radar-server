# radar-server
server for the radar viewer controler senior project
## Dependancies
install rust via rustup
install cross:
`cargo install cross`
### Building for x86\_64-unknown-linux
`cross build --release`
### Building for aarch64-unknown-linux
`cross build --target aarch64-unknown-linux-musl --release`
### Building for x86\_64 Windows
`cross build --target x86_64-pc-windows --release`
