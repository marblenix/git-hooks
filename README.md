# git-hooks

Some helpful git hooks

## Install

### From Source

This method can be updated automatically using git, but requires building new versions for each update.

```bash
cargo install --root ~/.git-hooks --git https://github.com/marblenix/git-hooks.git pre-push prepare-commit-msg

git config --global core.hooksPath ~/.git-hooks/bin
```

Then, whenever you wish to update:

```bash
cargo install --force --root ~/.git-hooks --git https://github.com/marblenix/git-hooks.git pre-push prepare-commit-msg
```

### From Release

```bash
mkdir ~/.git-hooks
git config --global core.hooksPath ~/.git-hooks
```

Download the release file for your platform and manually place it in the ~/.git-hooks directory.

Then, whenever you wish to update; re-download the latest version from the release page and replace the file in the
~/.git-hooks directory.

## Usage

## Configuration

All configurations use the default git-config settings model. `git config --global` and `git config` can be used to set
all of these keys and values globally or per-project.

### Pre-Push

Key | Default | Possible Values | Effect
--- | ------- | --------------- | ------
hooks.pre-push.enabled | true | true, false| enable/disable the pre-push binary
hooks.pre-push.protectedBranches | "master,develop" | any string with values separated by "," | the branches listed will stop `git push` if you try to update that branch
hooks.prepare-commit-msg.enabled | true | true, false| enable/disable the prepare-commit-msg binary
hooks.prepare-commit-msg.branchSeparator | "&#124;" (vertical bar) | any string | separates the current branch from the commit message
