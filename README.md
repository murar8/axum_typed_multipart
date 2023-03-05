# axum_typed_multipart

[![](https://img.shields.io/crates/v/axum_typed_multipart.svg)](https://crates.io/crates/axum_typed_multipart)
[![](https://docs.rs/axum_typed_multipart/badge.svg)](https://docs.rs/axum_typed_multipart)
[![](https://github.com/murar8/axum_typed_multipart/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/murar8/axum_typed_multipart/actions/workflows/ci.yml)

Helper library for the [axum framework](https://github.com/tokio-rs/axum) designed to allow you to parse the `multipart/form-data` body of the supplied request into an arbitrary struct.

## Documentation

Documentation and installation instructions are available on [docs.rs](https://docs.rs/axum_typed_multipart)

## Release process

Direct push to the `main` branch is not allowed, any updates require a pull request to be opened. After all status checks pass the PR will be eligible for review and merge.

If a [SemVer](https://semver.org/) compatible git tag is pushed to the repo a new version of the package will be published to [crates.io](https://crates.io/crates/axum_typed_multipart).

## Improvements

- Allow populating optional fields using the `std::default::Default` implementation for the type.
- Allow for setting an arbitrary default value for optional fields.

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
