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

A change to a component (nv-rs, nvmod-rs, loot-rs, lodwright-rs) is made in that component's
repository first and reaches this repository as a submodule pointer bump. The full submodule
workflow is documented here once the submodules land (Phase 1 of the plan).

Security vulnerabilities must be reported privately as described in `SECURITY.md`.
