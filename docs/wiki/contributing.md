<!-- SPDX-License-Identifier: GPL-3.0-only -->
# Contributing

The project is owned by Joris Jonkers and is licensed GPL-3.0-only. Outside contributions are not accepted unless the
owner asks for them, and public pull requests may be closed without review. The rules are in
[CONTRIBUTING.md](../../CONTRIBUTING.md).

If you were invited:

- Keep changes small and tied to one issue.
- Do not include secrets, private data, game files, mod archives or local scratch files.
- Do not edit `CHANGELOG.md` by hand; the release tool writes it.
- Use Conventional Commits for every commit and pull request title.
- Include the tests or the check output in the pull request.

## Parts are separate repositories

The five [Components](components.md) live under `components/` as submodules. A change to a part is a pull request in
that part's repository. It reaches raul as a version bump.

## This site

The pages you are reading are Markdown files in `docs/wiki/` and are changed through pull requests like code. The
[demo](https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/demo/) comes from `docs/design/demo/`, and the [tour](https://jorisjonkers-dev.github.io/rauls-trusty-toolbox/tour/) from `site/`. A change to any of them is
published when it reaches the main branch. `python3 site/build.py --out <folder>` builds the whole site on your computer.

## Security

Report security problems as described in [SECURITY.md](../../SECURITY.md).

## Behaviour

Be kind. See the [code of conduct](../../CODE_OF_CONDUCT.md).
