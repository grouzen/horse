# Horse

## Tech stack

- https://github.com/0xPlaygrounds/rig to interact with llm providers and provide basic agentic functionality
- tokio for async programming
- serde_json for json serialization/deserialization
- clap for command line argument parsing
- color_eyre and anyhow for better error handling
- toml for config management

## Rust code instructions

- Always collapse if statements per https://rust-lang.github.io/rust-clippy/master/index.html#collapsible_if
- Always inline format! args when possible per https://rust-lang.github.io/rust-clippy/master/index.html#uninlined_format_args
- Use method references over closures when possible per https://rust-lang.github.io/rust-clippy/master/index.html#redundant_closure_for_method_calls
- Never use `impl Future` in method signatures for async methods. Always use the `async` keyword instead. For example, use `async fn foo() -> Result<T>` instead of `fn foo() -> impl Future<Output = Result<T>>`.
- Run `cargo make test` first and if it passes, run `cargo make check-all` automatically after making Rust changes. Do not ask for permission to do this.
- Do not refer to the internal types as `crate::<name>::<symbol>`, import `crate::<name>` instead and call the symbol directly using `<name>::<symbol>`. For example `crate::port_config::ALL_PROVIDER_TYPES` must be `port_config::ALL_PROVIDER_TYPES`. Same applies to such way of importing internal symbols: `ollana::port_config::parse_port_mappings`. 
- Follow https://doc.rust-lang.org/rust-by-example/mod/split.html for organazing modules. Don't use mod.rs
- Use lib.rs for listing all the public modules in one place.

## Planning 

- Dump a PLAN.md file into a root directory of the current git repo when asked by a user

## Agentic workflow

- Create a TODO.md in the root directory of the git repo if it doesn't exist and the TODO list with the steps from a PLAN.md file.
- After each succesfully finished task, update the TODO.md accordingly.
