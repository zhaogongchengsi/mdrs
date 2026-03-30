# Writ integration feasibility spike

Date: 2026-03-30

## Goal

Attempt direct integration of `writ` into the current `mdrs` codebase to validate a Typora-style
"render-and-edit in one place" workflow.

## What was tried

1. Added `writ = "0.12.0"` in `Cargo.toml`.
2. Ran `cargo fetch` to resolve dependencies.

## Result

Dependency resolution failed because of a hard native-link conflict:

- `gpui-component = 0.5.1` depends on `tree-sitter = ^0.25.4`
- `writ = 0.12.0` depends on `tree-sitter = ^0.26`

Both crates link to the same native `tree-sitter` library (`links = "tree-sitter"`), and Cargo
allows only one version in the final dependency graph.

So **writ cannot be directly integrated** into the current stack without dependency unification.

## Runnable spike

To still verify writ behavior quickly, a separate isolated crate is added:

- `experiments/writ_spike`

It has its own dependency graph (`gpui` + `writ`) and avoids the `gpui-component` conflict in the
main app crate.

Run:

```bash
cargo run --manifest-path experiments/writ_spike/Cargo.toml
```

## Viable paths forward

### Path A (recommended): remove `gpui-component` usage and migrate UI primitives

- Replace title bar/buttons/input wrappers from `gpui-component` with GPUI/native components.
- Then integrate `writ` (which already targets GPUI).
- Cost: medium/high migration cost, but best long-term path for WYSIWYG goals.

### Path B: fork `writ` and downgrade to tree-sitter 0.25

- Create a maintained fork of writ and attempt dependency backport.
- Risk: API/runtime incompatibility; high maintenance burden.
- Cost: high uncertainty.

### Path C: self-build block editor on current stack

- Keep `gpui-component` and implement block-level WYSIWYG behavior in-app.
- Highest engineering effort, but full control.

## Cost estimate (engineering time)

- Path A (migrate + integrate writ): 20–40 person-days.
- Path B (fork writ): 15–35 person-days with high technical risk.
- Path C (self-build): 40–90 person-days depending on feature completeness.
