# Publishing Packages

Source:

- https://docs.atopile.io/atopile-0.14.x/guides/publish.md

Publish reusable packages to:

- https://packages.atopile.io/

## Package Quality

Good packages should include:

- Useful `ato.yaml` package metadata.
- A clear `README.md`.
- A working layout where applicable.
- Published source in GitHub.

The package index detects `README.md` and displays it.

## Package Shapes

Common repository shapes:

- Single package repository, where the repo name matches the package.
- Monorepo with multiple separate Atopile projects/packages.

## Package Metadata

The first segment of `package.identifier` must be the GitHub username or organization name.

Single-package example:

```yaml
package:
  identifier: atopile/some-package
  repository: https://github.com/atopile/some-package
  authors:
    - name: Pepper
      email: pepper@atopile.io
  summary: Short package summary.
  license: MIT
  homepage: https://github.com/atopile/some-package
```

Monorepo example:

```yaml
package:
  identifier: atopile/some-package
  repository: https://github.com/atopile/packages
  version: "0.1.0"
  authors:
    - name: Pepper
      email: pepper@atopile.io
  summary: Short package summary.
  license: MIT
  homepage: https://github.com/atopile/packages
```

Metadata fields:

| Field | Purpose |
| --- | --- |
| `package.identifier` | Unique `{owner}/{name}` package identifier. |
| `package.repository` | GitHub repository URL. |
| `package.version` | Optional semver package version. |
| `package.authors` | Author list with name and email. |
| `package.summary` | Short description. |
| `package.license` | Package license. |
| `package.homepage` | Documentation or project homepage. |

Version restrictions:

- Valid semver.
- No leading `v`.
- No pre-release or dev tags.
- No build metadata.

## GitHub Action

Publishing is currently supported through:

- https://github.com/atopile/publish-package

The workflow needs GitHub OIDC permissions:

```yaml
permissions:
  contents: write
  packages: write
```

For a single package, the docs recommend publishing on GitHub release creation:

```yaml
on:
  release:
    types: [created]

jobs:
  release:
    if: github.event.release.draft != true && github.event.release.prerelease != true
    permissions:
      contents: write
      packages: write
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: atopile/publish-package@main
```

For a monorepo, create a separate Atopile project per package, version each package in its own `ato.yaml`, and use the action's package entrypoint input.
