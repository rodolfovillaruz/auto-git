# watch-and-commit (`wac`)

A small Rust CLI that watches your working directory for file changes and
automatically stages, commits, and pushes them to Git — no manual `git add` /
`git commit` required.

It's built for scenarios where you want a continuous, low-friction commit
history: pairing with an AI coding agent, screen-recording a live coding
session, or just auto-saving work-in-progress to a branch as you edit.

## Features

- **Recursive filesystem watching** of the current directory using
  [`notify`](https://crates.io/crates/notify).
- **Debounced commits** — bursts of file events (e.g. a save that touches many
  files, or a build that writes temp files) are collected for 3 seconds and
  committed as a single batch, rather than one commit per event.
- **Pre-flight safety checks** before watching starts, so `wac` never commits
  on top of a messy or out-of-sync repo:
  - confirms the current directory is a Git repository,
  - refuses to start if there are pre-existing staged or unstaged changes,
  - refuses to start if there are untracked files,
  - fetches from the remote (if one is configured) and verifies `HEAD` is in
    sync with its upstream branch, reporting how many commits you are ahead,
    behind, or diverged.
- **Automatic push** — after each commit, `wac` pushes to the remote if one is
  configured; if not, it simply commits locally.
- **Bare-repo auto-initialisation** — if `GIT_WORK_TREE` and `GIT_DIR` are set
  and point at a directory/repo that doesn't exist yet, `wac` creates the work
  tree and initialises the bare repository for you.
- **No-op safe** — if a debounced batch of events doesn't actually change the
  index (e.g. a file was written back to its original contents), `wac` skips
  the commit instead of creating an empty one.

## Installation

### Cargo

```sh
cargo install watch-and-commit
```

### cargo-binstall

Pre-built binaries are published on GitHub Releases and can be fetched
directly, skipping compilation:

```sh
cargo binstall watch-and-commit
```

### Homebrew (macOS/Linux)

```sh
brew install rodolfovillaruz/tap/watch-and-commit
```

### Winget (Windows)

```sh
winget install RodolfoVillaruz.WatchAndCommit
```

### Debian/Ubuntu (`.deb`)

Download the `.deb` asset from the
[latest release](https://github.com/rodolfovillaruz/watch-and-commit/releases)
and install it:

```sh
sudo dpkg -i wac_*.deb
```

### From source

```sh
git clone https://github.com/rodolfovillaruz/watch-and-commit.git
cd watch-and-commit
cargo build --release
# binary is at target/release/wac
```

All install methods produce a binary named **`wac`**.

## Usage

Run it from inside the Git repository you want to auto-commit:

```sh
wac
```

`wac` will:

1. Run pre-flight checks (clean working tree, no untracked files, in sync
   with upstream if a remote exists).
2. Start watching the current directory (`.`) recursively.
3. On file activity, wait for a 3-second quiet period, then run:
   ```sh
   git add .
   git commit -m "Update"
   git push   # only if a remote is configured
   ```
4. Repeat until you stop it with `Ctrl+C`.

If the pre-flight checks fail (dirty tree, untracked files, or a
diverged/behind/ahead branch), `wac` prints the reason and exits without
watching anything — commit, stash, push, or pull as needed and re-run it.

### Using a detached work tree

You can watch a directory while storing Git metadata in a separate bare
repository via the standard Git environment variables:

```sh
export GIT_DIR=/path/to/bare/repo.git
export GIT_WORK_TREE=/path/to/watched/files
wac
```

If either path doesn't exist yet, `wac` creates the work tree directory and
initialises a bare repository at `GIT_DIR` automatically before watching
starts.

## How it works

The project is a small `tokio` async binary split into three modules:

- **`main.rs`** — runs the pre-flight checks, sets up the `notify` watcher,
  and wires filesystem events into an `mpsc` channel (capacity 100) consumed
  by the debouncer. Also handles optional bare-repo initialisation via
  `GIT_WORK_TREE` / `GIT_DIR`.
- **`debouncer.rs`** — receives events from the channel and coalesces bursts
  of activity: it waits for the first event, then keeps collecting further
  events (resetting a 3-second timer each time) until things go quiet, then
  hands the whole batch to the event handler.
- **`event_handler.rs`** — logs a human-readable summary of each event in the
  batch (created/modified/renamed/removed files), then runs `git add .`,
  checks whether the index actually changed, and if so commits with the
  message `"Update"` and pushes if a remote is configured.

Filesystem "access" events (e.g. a file merely being read) are filtered out
at the watcher level so they don't trigger commits.

## Caveats

- `git add .` stages **all** changes in the work tree — rely on `.gitignore`
  for anything that shouldn't be committed.
- Every commit uses the fixed message `"Update"`; there's no diff
  summarisation.
- Designed for personal/scratch workflows (notes repos, pairing sessions,
  auto-saved WIP branches) — not intended for shared, production, or
  protected branches.

## Development

A `Makefile` wraps the common Cargo workflows:

```sh
make fmt        # cargo fmt --all
make lint       # cargo clippy -- -D warnings
make test       # cargo test
make build      # cargo build --release
make check      # fmt + lint + test + build
make verify     # cargo verify-project
make dry-run    # check + verify + cargo publish --dry-run
make clean      # cargo clean
```

Releases are cut with `make tag` (tags and pushes `vX.Y.Z`) followed by
`make publish` (dry-run, tag, then `cargo publish`). Pushing a `v*` tag
triggers `.github/workflows/release.yml`, which creates a GitHub Release,
uploads cross-platform binaries and a `.deb` package, and updates the
Homebrew tap and Winget manifest.

## License

MIT — see [LICENSE](LICENSE).
