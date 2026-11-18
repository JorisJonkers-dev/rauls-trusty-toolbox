# Contributing

This project is owned by Joris Jonkers. It is licensed GPL-3.0-only (see `LICENSE`).

External contributions are not accepted unless Joris Jonkers explicitly asks for them. Public
pull requests may be closed without review.

If you were invited to contribute:

1. keep changes scoped to the requested repository and issue
2. do not include secrets, private data, game files, mod archives, local scratch files, or
   generated planning artifacts
3. do not hand-edit `CHANGELOG.md`; release-please owns changelog updates
4. include the relevant tests or validation output in the pull request
5. use Conventional Commits for every commit and pull request title, with impersonal,
   professional wording

## Components are submodules

The five Components live under `components/` as git submodules pinned to the versions they
were released at, and the lock files fix the Rust crates. Each keeps its own repository, history, issues, CI and
releases. A change to a Component is a pull request in that Component's repository; it reaches
this repository only as a pointer bump.

| Component | Folder | Branch the pointer must be on |
| --- | --- | --- |
| nv-rs | `components/nv-rs` | `fork` (or a release tag) |
| nvmod | `components/nvmod-rs` | `main` (or a release tag) |
| loot-rs | `components/loot-rs` | `main` (or a release tag) |
| lodwright-rs | `components/lodwright-rs` | `main` (or a release tag) |
| ttwright-rs | `components/ttwright-rs` | `main` (or a release tag) |

### Clone

```sh
git clone --recursive git@github.com:JorisJonkers-dev/rauls-trusty-toolbox.git
# or, in an existing clone:
git submodule update --init --recursive
```

### Work on a Component

A submodule is checked out at a commit, not on a branch (a detached HEAD). Never commit there:
the commit belongs to no branch and is lost on the next `git submodule update`. Work in the
Component's own clone instead (`~/Projects/JorisJonkers-dev/<repo>`), on a branch, through a pull
request in that repository. If you must work inside `components/<repo>`, first run
`git switch -c <type>/<topic>` there.

### Bump a pointer

`components/<repo>.version` holds the release the submodule is at; the Component follower reads it, and CI fails when it differs from the submodule.

After the Component's release is out:

```sh
scripts/bump-component.sh <component> <tag>
cargo update --workspace
cargo update --workspace --manifest-path crates/raul-gui/Cargo.toml
git add components/<repo> components/<repo>.version Cargo.lock crates/raul-gui/Cargo.lock
git commit -m "chore(deps): bump <repo> to vX.Y.Z"
```

A bump changes the submodule pointer and the lock files only. One pull request moves one Component,
and it merges once the full pipeline and the end-to-end run pass on it.

Security vulnerabilities must be reported privately as described in `SECURITY.md`.
