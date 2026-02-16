# Nexum

**/ˈnɛk.səm/** (NEK-sum)

Nexum is a NixOS-based operating system for parallel, agentic software development.  
It makes project context a first-class OS primitive so you can recover the right terminal, editor, browser, and state in seconds.

## Vision

Modern agent workflows break when parallelism grows. Nexum solves this by introducing **project capsules** with:

- one project identity
- one resource namespace
- one attention channel
- one restore action

Core loop:

`signal -> identify project -> restore full context -> act`

## What Nexum Targets

- Fast context restoration under heavy multitasking
- Fewer port, cookie, and OAuth collisions
- Better supervision of long-running agent tasks
- Stable project-focused workspace behavior

## Architecture Baseline

- **Base distro:** NixOS
- **Window manager layer:** niri-centric shell with deep Stead integration
- **Capsule isolation:** hybrid (host-default + isolated mode)
- **Identity strategy:** hybrid (domain isolation + profile fallback)
- **Stead integration:** OS control-plane-first integration
- **Isolated-mode backend:** native Nix shell (`nix develop`)
- **Routing strategy:** custom routing daemon
- **Migration strategy:** shadow mode + feature-flag cutovers

## Current Status

Active implementation with stacked milestone PRs and TDD-first contracts.
Current work is focused on control-plane surfaces (capsule lifecycle, routing, restore orchestration, cutover gates, supervisor status, and Stead batch ingress).

## Design Principles

- **Restoration first:** interruption without context restore is failure
- **Parallel by default:** multiple active projects should feel normal
- **Predictable over flashy:** stable behavior beats frequent novelty
- **Safe rollout:** new architecture paths ship behind flags with rollback

## License

MPL-2.0

## Name

**Nexum** — a binding/connection.  
The name reflects the goal: binding project state, tools, and attention into one coherent system.
