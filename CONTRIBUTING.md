# Contributing

Thank you for considering to contribute to StableView!

When contributing to this repository, please first discuss the change you wish to make via [discuss](<(https://github.com/shubhamai/StableView/discussions)>), [issue](https://github.com/shubhamai/StableView/issues/new/choose), email, or any other method with the owners of this repository before making a change.

Please note we have a [code of conduct](CODE_OF_CONDUCT.md), please follow it in all your interactions with the project.

## Development environment setup

To set up a development environment, please follow these steps:

1. Install [Rust](https://www.rust-lang.org/).

2. Clone the repo

   ```sh
   git clone https://github.com/shubhamai/StableView
   ```

3. Install [opencv-rust](https://github.com/twistedfall/opencv-rust) on your system.

4. Run `cargo run` to run the application without any optimizations. To run the application fully optimized, add `--release` to the command, ie. `cargo run --release`

5. To build the `.msi` installer for windows -
   1. Install [WiX Toolset v3.14.1](https://github.com/wixtoolset/wix3/releases/).
   2. Install [cargo-wix](https://github.com/volks73/cargo-wix).
   3. Run `cargo wix`. A new folder will be created in `target` folder containing the `.msi` file.

### Apple Silicon 

To build stableview on apple silicon

`sudo ORT_LIB_LOCATION=./assets/onnx_osx ORT_STRATEGY=system DYLD_FALLBACK_LIBRARY_PATH=./assets/onnx_osx/lib cargo run --release`

## Issues and feature requests

You've found a bug in the source code, a mistake in the wiki or maybe you'd like a new feature? Take a look at [GitHub Discussions](https://github.com/shubhamai/StableView/discussions) to see if it's already being discussed. You can help us by [submitting an issue on GitHub](https://github.com/shubhamai/StableView/issues/new/choose). Before you create an issue, make sure to search the issue archive, your issue may have already been addressed!

### How to submit a Pull Request

1. Search our repository for open or closed
   [Pull Requests](https://github.com/shubhamai/StableView/pulls)
   that relate to your submission. You don't want to duplicate effort.
2. Fork the project
3. Create your feature branch (`git checkout -b feat/amazing_feature`)
4. Commit your changes (`git commit -m 'feat: add amazing_feature'`).
5. Push to the branch (`git push origin feat/amazing_feature`)
6. [Open a Pull Request](https://github.com/shubhamai/StableView/compare?expand=1)
