# Prints the list of recipes.
default:
    @just --list

# Builds the whole project.
build:
  cargo build

# Build all schemas
schemas:
  scripts/build_schemas.sh

# Tests the whole project.
test:
  cargo test

# Alias to the format recipe.
fmt:
  @just format

# Formats the rust, toml and sh files in the project.
format:
  cargo fmt --all
  find . -type f -iname "*.toml" -print0 | xargs -0 taplo format
  find . -type f -name '*.sh' -exec shfmt -w {} \;
  scripts/utils/format_md.sh

# Runs clippy with the a feature flag if provided.
lint:
  cargo clippy --all -- -D warnings

# Tries to fix clippy issues automatically.
lintfix:
  cargo clippy --fix --allow-staged --allow-dirty --all-features
  just format

# Checks the whole project with all the feature flags.
check-all:
  cargo check --all-features

# Cargo check.
check:
  cargo check

# Cargo clean and update.
refresh:
  cargo clean && cargo update

# Cargo watch.
watch:
  cargo watch -x lcheck

# Watches tests with the a feature flag if provided.
watch-test FEATURE='':
  cargo watch -x "nextest run"

# Compiles and optimizes the contracts.
optimize:
  scripts/build_release.sh

# Prints the artifacts versions on the current commit.
get-artifacts-versions:
  scripts/get_artifacts_versions.sh

# Prints the artifacts size. Optimize should be called before.
get-artifacts-size:
  scripts/check_artifacts_size.sh

# Installs the env loader locally.
install-env-loader:
    scripts/deployment/deploy_env/add_load_chain_env_alias.sh
