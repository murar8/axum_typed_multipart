# axum_typed_multipart

[![](https://img.shields.io/crates/v/axum_typed_multipart.svg)](https://crates.io/crates/axum_typed_multipart)
[![](https://docs.rs/axum_typed_multipart/badge.svg)](https://docs.rs/axum_typed_multipart)
[![.github/workflows/release.yml](https://github.com/murar8/axum_typed_multipart/actions/workflows/release.yml/badge.svg)](https://github.com/murar8/axum_typed_multipart/actions/workflows/release.yml)
[![.github/workflows/audit.yml](https://github.com/murar8/axum_typed_multipart/actions/workflows/audit.yml/badge.svg)](https://github.com/murar8/axum_typed_multipart/actions/workflows/audit.yml)
[![codecov](https://codecov.io/gh/murar8/axum_typed_multipart/branch/main/graph/badge.svg?token=AUQ4P8EFVK)](https://codecov.io/gh/murar8/axum_typed_multipart)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Designed to seamlessly integrate with [Axum](https://github.com/tokio-rs/axum), this crate simplifies the process of handling `multipart/form-data` requests in your web application by allowing you to parse the request body into a type-safe struct.

## Documentation

Documentation and installation instructions are available on [docs.rs](https://docs.rs/axum_typed_multipart)

## Release process

When a [SemVer](https://semver.org/) compatible git tag is pushed to the repo a new version of the package will be published to [crates.io](https://crates.io/crates/axum_typed_multipart).

## Contributing

Direct push to the `main` branch is not allowed, any updates require a pull request to be opened. After all status checks pass the PR will be eligible for review and merge.

Commit messages should follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/#summary) specification.

The project comes with an optional pre-configured development container with all the required tools. For more information on how to use it please refer to <https://containers.dev>

To make sure your changes match the project style you can install the pre-commit hooks with `pre-commit install`. This requires [pre-commit](https://pre-commit.com/) to be installed on your system.

## License

Copyright (c) 2023 Lorenzo Murarotto <lnzmrr@gmail.com>

Permission is hereby granted, free of charge, to any person
obtaining a copy of this software and associated documentation
files (the "Software"), to deal in the Software without
restriction, including without limitation the rights to use,
copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the
Software is furnished to do so, subject to the following
conditions:

The above copyright notice and this permission notice shall be
included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES
OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING
FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR
OTHER DEALINGS IN THE SOFTWARE.
