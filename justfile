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
  scripts/get_artifacts_versions.sh --skip-verbose

# Prints the artifacts size. Optimize should be called before.
get-artifacts-size:
  scripts/check_artifacts_size.sh

# Installs the env loader locally.
install-env-loader:
    scripts/deployment/deploy_env/add_load_chain_env_alias.sh

# Deploys MANTRA Dex on the given CHAIN, default is mantra-testnet.
deploy CHAIN='mantra-testnet' CONTRACT='all':
    ./scripts/deployment/deploy_mantra_dex.sh -c {{CHAIN}} -d {{CONTRACT}}

# Stores the MANTRA Dex contracts on the given CHAIN, default is mantra-testnet.
store CHAIN='mantra-testnet' CONTRACT='all':
    ./scripts/deployment/deploy_mantra_dex.sh -c {{CHAIN}} -s {{CONTRACT}}

# Deploys a pool on MANTRA Dex on the given CHAIN, default is mantra-testnet.
deploy-pool CHAIN='mantra-testnet' POOL_FILE='pool.json' AMOUNT_DENOM_0='0' AMOUNT_DENOM_1='0':
    @# Ensure package.json exists, install dependencies if needed, then run the script
    @[ -f package.json ] || npm init -y; \
    [ -d node_modules ] || npm install @cosmjs/encoding @cosmjs/proto-signing @cosmjs/cosmwasm-stargate dotenv; \
    node scripts/deployment/deploy_pool.js {{CHAIN}} {{POOL_FILE}} {{AMOUNT_DENOM_0}} {{AMOUNT_DENOM_1}}

# Installs pre-commit git hooks.
install-git-hooks:
    ./scripts/hooks/commit-msg.sh --install
    ./scripts/hooks/pre-commit.sh --install

# ⚠️ Toggles OFF all features (deposits, withdrawals, swaps) for all pools in a Pool Manager.⚠️
emergency-toggle-pool-features-off:
    scripts/emergency/toggle_pool_features.sh

# ⚠️ Closes all farms in the Farm Manager.⚠️
emergency-close-farms:
    scripts/emergency/close_farms.sh