# Contributing to Atupa

We are thrilled that you want to contribute to Atupa! Whether you are fixing a bug, improving the visualization, or adding a new protocol adapter, your help is appreciated.

## Development Workflow

1.  **Fork the repo**: Create your own copy of the repository.
2.  **Clone it**: `git clone git@github.com:One-Block-Org/Atupa.git`
3.  **Create a branch**: `git checkout -b feature/cool-new-thing`
4.  **Make changes**: Implement your logic.
5.  **Run tests**: `cargo test --workspace`
6.  **Check lints**: `cargo clippy --workspace` & `cargo fmt --all -- --check`
7.  **Submit a PR**: Open a Pull Request from your branch to our `main`.

## Code Standards

-   **Documentation**: Please document all public structs and functions using standard `///` doc comments.
-   **Tests**: All new features must include unit tests. Use `atupa-parser`'s existing tests as a template for trace-aggregation logic.
-   **Structure**: Keep the library and CLI logic separate.
    -   `atupa-core`: Shared types.
    -   `atupa-parser`: Pure trace transformation logic.
    -   `atupa-sdk`: Unified developer library.
    -   `bin/atupa`: User-facing binary.
    -   `crates/atupa-<name>`: Specialized protocol adapters.

## 💉 Adding a Protocol Adapter

Protocol adapters allow Atupa to understand the high-level business logic of a specific DeFi protocol. To add a new one:

1.  **Create a crate**: Use `crates/atupa-lido` as a template.
2.  **Implement `ProtocolAdapter`**: Define how to extract string labels and metrics from an EVM frame.
3.  **Export via SDK**: Add your crate to `crates/atupa-sdk` to make it accessible to library users.
4.  **CLI Integration**: Update `bin/atupa/src/main.rs` to include your adapter in the `audit` command.

## License

By contributing, you agree that your contributions will be licensed under the project's [MIT](LICENSE-MIT) and [Apache 2.0](LICENSE-APACHE) licenses.
