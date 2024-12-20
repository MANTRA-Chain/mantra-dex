# Contributing to MANTRA Dex

====================

Thank you for considering contributing to MANTRA Dex. We appreciate your help in making our project better. This document
outlines the steps to contribute to our repository.

### Prerequisites

To download the necessary tools, clone the repository and so on, you need network access.

Additionally, the following are the tools you'll need:

- [Git](https://git-scm.com/)
- [Rust](https://rustup.rs/)
- wasm32 target set for Rust
- [PyO3](https://pyo3.rs), which requires a Python installation **with a shared library** (see https://github.com/PyO3/pyo3/tree/main#using-python-from-rust for example instructions on installing this for Ubuntu)
- [Just](https://just.systems/man/en/)

```bash
$ rustup default stable
$ cargo version
# If this is lower than 1.80.0, update
$ rustup update stable

$ rustup target list --installed
$ rustup target add wasm32-unknown-unknown

# for generating contract
$ cargo install cargo-generate --features vendored-openssl
$ cargo install cargo-run-script
```

## Getting Started

To start contributing with the codebase, follow these steps:

- Fork the repository: Create a fork of the MANTRA Dex repository to your own GitHub account.
- Clone the repository: Clone the forked repository to your local machine using `git clone git@github.com:<your-username>/mantra-dex.git`.
- Install dependencies: Install the required dependencies for the project, `rust`, `just`.
- Install pre-commit hooks: Install the pre-commit hooks using `just install-git-hooks`.
- Create a new branch: Create a new branch for your changes using `git checkout -b your-branch-name`.
- Make changes: Make the necessary changes to the code.
- Format the code: Format the code using `just fmt`.
- Commit changes: Commit your changes using `git add .` and `git commit -m "your-commit-message"`. Make sure you follow
  the [conventional commit message format](https://www.conventionalcommits.org/). The pre-commit hooks should prevent you
  making mistakes here.
- Push changes: Push your changes to your forked repository using `git push origin your-branch-name`.
- Create a pull request: Create a pull request from your forked repository to the main MANTRA Dex repository.

## Code Review

All pull requests will be reviewed by the maintainers. We may ask for changes or improvements to be made before merging.

Bear in mind any changes should be tested and documented. If you're adding a new feature, make sure to add tests for it,
following the existing patterns in the codebase, i.e. use the existing `TestSuite`.

## License

By contributing to MANTRA Dex, you agree that your contributions will be licensed under the [MPL-2.0](../LICENSE).

## Code of Conduct

Please read and follow our [Code of Conduct](CODE_OF_CONDUCT.md).

## Reporting Security Issues

To report a security issue, please follow our [Security Policies and Procedures](SECURITY.md).
