# Contributing to marg

Thank you for helping. marg is MIT licensed; a contribution is accepted under the
same license (see [LICENSE](LICENSE)).

## Issues

Open an issue on GitHub for a bug or a feature request. For a bug, give the marg
version, the features it is built with (`postgres`, `sqlite`, `token`, `key`,
`keep_env_secret`, `gen_secret`, `--no-default-features`), the arguments, environment
variables and config file it was given (with any secret taken out), what you expected,
and what you got.

A security problem is not reported in a public issue: write to
**support@dbinvent.com** instead.

## Pull requests

1. Fork the repository and branch from `main`.
2. Keep a change to one subject; describe what it changes and why.
3. Build and test it (below); add tests for what you change (`src/tests.rs`).
4. Open the pull request against `main`.

## Build and test

```
cargo test                                          # default features (postgres)
cargo test --no-default-features                    # no db feature
cargo test --no-default-features --features sqlite  # sqlite
cargo test --features sqlite,key,keep_env_secret,gen_secret
cargo check --all-features --all-targets            # token too: its tests run a command
cargo llvm-cov --features sqlite,key,keep_env_secret,gen_secret --summary-only   # CI fails under 69% of lines
```

Before a pull request, run the checks as well (`cargo make audit`, `cargo make geiger`):

```
cargo audit      # known vulnerabilities in the dependencies
cargo geiger     # unsafe code in marg and its dependencies
```

marg's one unsafe call is `std::env::remove_var`, which takes the secret out of the
environment after it is read (unsafe since edition 2024; the `keep_env_secret` feature
leaves it there instead). A change does not add another.

## Code of conduct

Taking part in this project means following its
[Code of Conduct](CODE_OF_CONDUCT.md).
