---
name: push-release
description: Bumps the version, commits, tags, and pushes to trigger a GitHub release. Use when the user says "push and release", "release", "bump and release", or similar.
---

# Push and Release

Bump the project version, commit all changes, tag, and push to trigger the GitHub Actions release workflow (`.github/workflows/release.yml`).

## Step 1: Determine the version bump

Read `Cargo.toml` and parse the current `version` field.

Decide which semver component to bump:

| User says | Bump | Example |
|---|---|---|
| _(nothing specific)_ | **minor** | `0.2.0` → `0.3.0` |
| "patch" / "bugfix" | **patch** | `0.2.0` → `0.2.1` |
| "major" | **major** | `0.2.0` → `1.0.0` |
| an explicit version (e.g. "make it 1.0.0") | use that version exactly | — |

When bumping, always reset the components to the right of the bumped component to zero (e.g. minor bump: `0.2.3` → `0.3.0`).

## Step 2: Update `Cargo.toml`

Edit the `version = "..."` line in `Cargo.toml` to the new version.

## Step 3: Build and verify

Run:

```bash
cargo build
```

This accomplishes three things:
1. Validates the project still compiles.
2. Updates `Cargo.lock` with the new version automatically.
3. Bakes the new version into the binary via `env!("CARGO_PKG_VERSION")`.

Then verify the binary reports the correct version:

```bash
./target/debug/dbloada --version
```

Expected output: `dbloada <new_version>`. If it does not match, stop and investigate.

## Step 4: Check git state

Run `git status` and `git diff` to review what will be committed. Ensure there are no unintended changes. If there are untracked files that look unrelated, ask the user before staging them.

## Step 5: Commit

Stage and commit. Always include `Cargo.toml` and `Cargo.lock`. Also include any other modified tracked files that are part of this release.

```bash
git add Cargo.toml Cargo.lock <other changed files>
```

Write a commit message following the **Conventional Commits** format. Choose the prefix based on what changed:

| Prefix | When to use |
|---|---|
| `feat:` | New features or capabilities were added |
| `fix:` | Bug fixes |
| `chore:` | Version bump only, no functional changes |
| `refactor:` | Code restructuring without behavior change |

If the commit includes a mix of changes, use the prefix that best describes the most significant change. Always mention the new version number in the message body or subject.

Example:

```
feat: add project validation and bump to v0.3.0

Add schema validation for dbloada.yaml project files.
```

If the release is a version-only bump with no other changes:

```
chore: release v0.3.0
```

## Step 6: Tag

Create an annotated-style lightweight tag with the `v` prefix:

```bash
git tag v<new_version>
```

Examples: `v0.3.0`, `v1.0.0`, `v0.2.1`.

## Step 7: Push

Push the commit and the tag:

```bash
git push && git push --tags
```

## Step 8: Confirm

After pushing, inform the user:

1. The new version number.
2. The tag that was pushed (e.g. `v0.3.0`).
3. That the release workflow has been triggered and will build binaries for the configured targets.
4. Provide the URL to watch the workflow: `https://github.com/PerArneng/dbloada/actions`

## Errors and edge cases

- **Dirty working tree with unrelated changes**: Ask the user whether to include them in the release commit or stash them first.
- **Build failure**: Stop. Show the error. Do not commit or tag.
- **Tag already exists**: Stop and tell the user. Suggest bumping to the next version instead.
- **Push fails**: Show the error. The local commit and tag are still intact — the user can retry manually or ask you to retry.
- **User asks for a pre-release** (e.g. `v1.0.0-rc.1`): Use the exact version string they provide. The release workflow triggers on any `v*` tag.
