# DevDoctor — Project Decisions

_A quick reference for future development so I don't forget._

## Goal

Build a **lazygit-style Rust TUI** that helps developers understand their local development environment: installed runtimes, PATH resolution, development processes, listening ports, and associated projects. Prioritize a useful, trustworthy tool over a feature-heavy prototype.

## Guiding decisions

- **Primary learning goal:** practical, idiomatic Rust and software architecture; Linux internals are a secondary benefit.
- **Use established crates** for solved infrastructure problems; write our own DevDoctor-specific inspection, correlation, filtering, and diagnostic logic.
- **Read-only first.** Never automatically terminate a process based on a heuristic; any future termination feature needs explicit confirmation and identity revalidation.
- **Cross-platform in design:** Linux, macOS, Windows eventually. Develop and test on Linux (CachyOS) first; tolerate missing or restricted system information.
- **Own our data models.** Convert crate data into DevDoctor structs so the UI does not depend directly on a particular crate.
- **Develop independently.** Define feature requirements, then implement and research solutions yourself. Code reviews should prioritize correctness, Rust idioms, performance, architecture, and rationale—not provide full solutions by default.

## Dependency decisions

| Area                               | Choice                                                       | When                              |
| ---------------------------------- | ------------------------------------------------------------ | --------------------------------- |
| Process inspection                 | `sysinfo`                                                    | **Now**                           |
| Executable lookup                  | `which`                                                      | When revisiting runtime inspector |
| Listening ports / owning processes | `listeners` (validate fit when integrating)                  | Ports milestone                   |
| TUI rendering                      | `ratatui`                                                    | TUI milestone                     |
| Terminal events                    | `crossterm`                                                  | TUI milestone                     |
| CLI argument parsing               | `clap`                                                       | When CLI flags/subcommands exist  |
| Error handling                     | `anyhow` / `thiserror`                                       | Only as needed                    |
| Git repository discovery           | Simple filesystem inspection first; evaluate `gix` if needed | Project milestone                 |
| Configuration                      | `directories` + `serde`                                      | Later, if persistence is needed   |

**Not needed for v0.1:** Tokio, a database, Docker SDKs, an HTTP client, or a plugin system. Add dependencies incrementally, not all upfront.

## Architecture

```text
TUI (views + input)
        ↓
Application state (snapshots + selection)
        ↓
DevDoctor inspectors / analysis
  runtimes · processes · ports · projects
        ↓
Crates + platform-specific OS APIs
```

Organize modules as features grow; don't create the entire eventual directory tree prematurely.

## Milestones

1. **Runtime inspector — prototype complete.** Find Python, Node, `rustc`, and Cargo via PATH; capture versions; represent missing values in a `Runtime` struct. Currently in `src/runtime.rs`.
2. **Process inspector — next.** Use `sysinfo` to enumerate processes and capture PID, command, working directory, and optionally parent/start time. Return your own structs; handle inaccessible/disappearing processes.
3. **Port inspector.** Discover listening ports and correlate them with PIDs/processes.
4. **First Ratatui TUI.** Views for runtimes, processes, and ports; keyboard navigation, details, refresh.
5. **Dev-specific intelligence.** Associate processes with Git projects, identify likely dev workloads, filter/search, cautiously flag _possibly_ stale processes.
6. **Polish / v1.** Tests, documentation, cross-platform validation, clearer diagnostics, safe user-confirmed process actions, and releases.

## Runtime-inspector polish backlog

- Replace hand-rolled PATH lookup with `which` after comparing behavior.
- Validate executability rather than relying on `.exists()`.
- Preserve version-check failures instead of discarding them with `.ok()`.
- Handle missing `PATH` without panicking.
- Simplify `HashSet` deduplication (`insert` can report whether an item was new).
- Avoid printing from inspection helpers; return errors/data to the caller.
- Unused-field/unused-function warnings are expected during prototype development.
