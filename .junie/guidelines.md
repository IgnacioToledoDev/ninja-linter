### Ninja Linter Development Guidelines

#### Build/Configuration Instructions
- **Rust Toolchain**: Use a recent stable Rust version (Edition 2024).
- **Compilation**: Standard `cargo build` command compiles the project.
- **Build Metadata**: Uses `shadow-rs` to embed build-time information (e.g., version, git branch).
- **Runtime Configuration**:
  - The tool uses a local config file: `.ninja-linter.json`.
  - If the file is missing, it will be created upon first use of the `--test` flag.
  - Key setting: `test_command` (e.g., `"docker exec ninja_symfony bin/phpunit"`).

#### Testing Information
- **Execution**: Run all tests using `cargo test`.
- **Targeted Testing**: Run specific tests using `cargo test <test_name_substring>`.
- **Adding Tests**:
  - Internal tests should be added to a `tests` module within the relevant `.rs` file using the `#[cfg(test)]` attribute.
  - For cross-module or integration tests, use the `tests/` directory if applicable.
- **Example Test**:
  ```rust
  #[cfg(test)]
  mod tests {
      #[test]
      fn test_example_logic() {
          let result = 2 + 2;
          assert_eq!(result, 4);
      }
  }
  ```

#### Additional Development Information
- **Docker Integration**: The tool is designed to interface with a Docker container named `ninja_symfony`. Most commands (`cs:fix`, `stan`) are executed inside this container via `docker exec`.
- **Command Handling**: `src/command.rs` contains the logic for interacting with external processes.
- **Exit Codes**:
  - `0`: Success.
  - `1`: Fatal Error or linting failure.
- **Logging**: Uses the `log` and `colored` crates for terminal output.
