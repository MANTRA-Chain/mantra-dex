#!/bin/bash
export PATH=$PATH:/usr/local/bin

#
# MANTRA commit hook, used to make sure commits follow the conventional commits format.
# See more at https://www.conventionalcommits.org/
#
# Install the hook with the --install option.
#

project_toplevel=$(git rev-parse --show-toplevel)
git_directory=$(git rev-parse --git-dir)

install_hook() {
	mkdir -p "${git_directory}/hooks"
	ln -sfv "${project_toplevel}/scripts/hooks/commit-msg.sh" "${git_directory}/hooks/commit-msg"

	echo "Checking required tools..."

	if ! command -v rustfmt >/dev/null 2>&1; then
		echo "Installing rustfmt..."
		rustup component add rustfmt
	fi

	if ! command -v taplo >/dev/null 2>&1; then
		echo "Installing taplo-cli..."
		cargo install taplo-cli --locked
	fi

	if ! command -v shfmt >/dev/null 2>&1; then
		echo "Warning: shfmt not found. Shell scripts won't be formatted."
		echo "Please install shfmt manually: https://github.com/mvdan/sh"
	fi
}

if [ "$1" = "--install" ]; then
	if [ -f "$git_directory/hooks/commit-msg" ]; then
		read -r -p "There's an existing commit-msg hook. Do you want to overwrite it? [y/N] " response
		case "$response" in
		[yY][eE][sS] | [yY])
			install_hook
			;;
		*)
			printf "Skipping hook installation :("
			exit $?
			;;
		esac
	else
		install_hook
	fi
	exit $?
fi

printf "Checking commit message format...\n"

COMMIT_MSG_FILE="$1"

if [ ! -f "$COMMIT_MSG_FILE" ]; then
	echo "Error: Commit message file not found."
	exit 1
fi

COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

# Regular expression to match Conventional Commits format
CONVENTIONAL_COMMIT_REGEX="^(feat|fix|docs|style|refactor|perf|test|build|ci|chore|revert)(\(\S+\))?: .+$"

if ! echo "$COMMIT_MSG" | grep -qE "$CONVENTIONAL_COMMIT_REGEX"; then
	echo "Commit message does not follow Conventional Commits format."
	echo "Example format: feat(scope): description"
	exit 1
fi
