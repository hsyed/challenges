# AGENTS.md

This repository is a KCL proof of concept for multi-tenant Kubernetes configuration.

## Purpose

The project demonstrates:

- shared platform policy in KCL
- tenant-specific configuration
- separate `dev` and `prod` environment profiles
- GitOps-friendly YAML rendering

The main reusable logic lives in:

- `modules/env.k`
- `modules/platform.k`
- `modules/tenant.k`

The concrete entrypoints live in separate package scopes under:

- `environments/dev/acme/main.k`
- `environments/dev/globex/main.k`
- `environments/prod/acme/main.k`
- `environments/prod/globex/main.k`

This directory-per-entrypoint layout is intentional. Do not collapse these files back into a single `environments/` directory with multiple sibling `.k` files using the same top-level symbol. That caused namespace/LSP collisions.

## Render Commands

Use these commands from the repo root:

```bash
XDG_CACHE_HOME=/tmp/kcl-cache HOME=/tmp kcl run environments/dev/acme/main.k -S manifests -o rendered/dev-acme.yaml
XDG_CACHE_HOME=/tmp/kcl-cache HOME=/tmp kcl run environments/dev/globex/main.k -S manifests -o rendered/dev-globex.yaml
XDG_CACHE_HOME=/tmp/kcl-cache HOME=/tmp kcl run environments/prod/acme/main.k -S manifests -o rendered/prod-acme.yaml
XDG_CACHE_HOME=/tmp/kcl-cache HOME=/tmp kcl run environments/prod/globex/main.k -S manifests -o rendered/prod-globex.yaml
```

Notes:

- `-S manifests` selects the top-level export from each entrypoint.
- `XDG_CACHE_HOME=/tmp/kcl-cache HOME=/tmp` is used because in this environment `kcl` tried to write cache under `~/.cache`, which was read-only inside the sandbox.

If the project is moved to another directory, these commands should still work as long as they are run from the new repo root.

## Important Structure Assumptions

1. This is a KCL module root.
   Required files:
   - `kcl.mod`
   - `kcl.mod.lock`

2. Imports assume module-root execution.
   Example:
   - `import modules.env as env`
   - `import modules.tenant as tenants`

3. Entry points should stay in isolated directories.
   Preferred pattern:
   - `environments/<environment>/<tenant>/main.k`

4. Top-level entrypoint export is `manifests`.
   Keep this stable unless you update README and render commands.

## Moving The Project

If this repository is moved:

- keep the repo root intact
- keep `kcl.mod` at the root
- run KCL commands from the repo root
- keep the relative paths under `modules/`, `environments/`, and `rendered/`

No absolute filesystem paths are required by the KCL code itself.

## Safe Changes

Safe refactors:

- adding more tenants under `environments/<env>/<tenant>/main.k`
- adding more reusable helpers in `modules/tenant.k`
- extending `modules/env.k` with more environment fields
- adding more rendered outputs under `rendered/`

Changes that need care:

- moving `kcl.mod`
- flattening entrypoint directories
- renaming `manifests`
- changing import roots

## Current Status

At the time this file was written, all four entrypoints rendered successfully with `kcl run ... -S manifests`.
