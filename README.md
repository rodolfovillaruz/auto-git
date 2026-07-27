# Async Git Auto-Commit File Watcher

A Rust-based file system watcher that automatically commits (and optionally pushes) changes to a Git repository whenever files are modified. Built with `tokio` for async I/O and `notify` for cross-platform file system events, it uses a debouncing strategy to batch rapid successive changes into a single commit.

---

## ✨ Features

- **Recursive file watching** — monitors the current directory and all subdirectories.
- **Smart debouncing** — waits for file system "event storms" to settle (default: 3 seconds) before committing, avoiding noisy commits during bulk operations like `npm install` or code formatting.
- **Auto-commit & push** — stages all changes, commits with a default message, and pushes to the configured remote (if any).
- **Pre-flight sanity checks** — verifies the repository is clean and in sync with its upstream before starting.
- **Bare repo auto-init** — if `GIT_DIR` and `GIT_WORK_TREE` are set and missing, they are created and initialised automatically.
- **Filters noise** — ignores `Access` events which would otherwise trigger unnecessary commits.

---

## 📦 Requirements

- [Rust](https://www.rust-lang.org/) (1.70+ recommended) with Cargo
- `git` available on your `PATH`
- A Git repository (or the ability to create one via `GIT_DIR`/`GIT_WORK_TREE` environment variables)

---
## 🚀 Installation

### From crates.io *(recommended)*

```bash
cargo install watch-and-commit
```

This downloads, compiles, and places the `wac` binary in `~/.cargo/bin/`.
Make sure that directory is on your `PATH` (the `rustup` installer adds it automatically).

```bash
wac --help
```

### With cargo-binstall (prebuilt, no compiling)

```bash
cargo binstall watch-and-commit
```

[`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall) fetches a prebuilt `wac` binary for your platform straight from the [GitHub Releases](https://github.com/rodolfovillaruz/watch-and-commit/releases) page instead of compiling from source.

### With Homebrew (macOS / Linux)

```bash
brew install rodolfovillaruz/tap/watch-and-commit
```

### With winget (Windows)

```powershell
winget install RodolfoVillaruz.WatchAndCommit
```

### Debian / Ubuntu (.deb)

Download the `.deb` for your architecture from the [latest release](https://github.com/rodolfovillaruz/watch-and-commit/releases/latest) and install it:

```bash
sudo dpkg -i wac_*.deb
```

### From source

Clone the repository and build:

```bash
git clone https://github.com/rodolfovillaruz/watch-and-commit
cd watch-and-commit
cargo build --release
```

The compiled binary will be available at `target/release/wac`.

### Dependencies

This project uses the following crates (add to your `Cargo.toml` if setting up from scratch):

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
notify = "8"
```

---

## 🏃 Usage

### Basic usage

From inside any Git repository:

```bash
cargo run --release
```

The watcher will:
1. Run pre-flight checks on the current working directory.
2. Start monitoring the current directory (`.`) recursively.
3. Debounce events and auto-commit whenever the dust settles.

Press **Ctrl+C** to stop.

### Using a detached work tree

You can watch a directory while storing the Git metadata in a separate bare repository by setting the standard Git environment variables:

```bash
export GIT_DIR=/path/to/bare/repo.git
export GIT_WORK_TREE=/path/to/watched/files
cargo run --release
```

If either directory does not exist, it will be created, and a bare repository will be initialised at `GIT_DIR` automatically.

---

## 🔄 How It Works

```
┌─────────────────┐    events     ┌──────────────┐    batched    ┌───────────────┐
│  notify Watcher │──────────────▶│  Debouncer   │──────────────▶│ Event Handler │
│  (file system)  │   (mpsc tx)   │ (tokio task) │  (3s window)  │  + git commit │
└─────────────────┘               └──────────────┘               └───────────────┘
```

1. **`notify::RecommendedWatcher`** produces events for any change in the watched directory.
2. Events are sent over a bounded `tokio::sync::mpsc` channel (capacity: 100).
3. The **debouncer** receives the first event, then keeps collecting events until a quiet period of `debounce_duration` (3 seconds) has elapsed.
4. The accumulated batch is handed off to the **event handler**, which:
   - Logs each event.
   - Runs `git add .`.
   - Checks if anything was actually staged (skips empty commits).
   - Runs `git commit -m "Update"`.
   - Runs `git push` (only if a remote is configured).

---

## 🛡️ Pre-flight Checks

Before the watcher starts, it runs a series of checks to ensure a sane starting state:

| # | Check                                                             | Failure behaviour |
|---|-------------------------------------------------------------------|-------------------|
| 0 | Auto-create work tree & init bare repo if env vars are set        | Errors if creation fails |
| 1 | Current directory is inside a Git work tree                       | Aborts            |
| 2 | No staged or unstaged tracked changes                             | Aborts            |
| 3 | No untracked (non-ignored) files                                  | Aborts (shows up to 10 files) |
| 4 | `git fetch` succeeds (only if a remote is configured)             | Aborts            |
| 5 | `HEAD` matches upstream tracking branch (ahead/behind/diverged)   | Aborts with details |

If no remote is configured, sync checks are skipped with a friendly notice.

---

## ⚙️ Configuration

Currently, configuration is done by editing `src/main.rs`:

- **Debounce window** — `Duration::from_secs(3)` in `main()`.
- **Watched path** — `Path::new(".")` in `main()`.
- **Commit message** — `"Update"` in `src/event_handler.rs::run_git_commit`.
- **Channel capacity** — `mpsc::channel(100)` in `main()`.

---

## 📂 Project Structure

```
src/
├── main.rs            # Entry point, pre-flight checks, watcher setup
├── debouncer.rs       # Async debouncing logic over an mpsc::Receiver
└── event_handler.rs   # Event pretty-printing and git operations
```

---

## ⚠️ Caveats

- **Force-commits everything**: `git add .` stages _all_ changes in the work tree. Use a good `.gitignore`.
- **Fixed commit message**: every commit is `"Update"`. No change summarisation (yet).
- **No rate limiting on push**: if you make rapid-fire edits on a slow connection, `git push` may queue up.
- **Not recommended for public-facing branches**: this tool is designed for personal notes, scratch repos, and "save every change" workflows — not for shared production branches.

---

## 🧪 Example Output

```
--- Async File Watcher with 3-Second Debounce ---
Fetching from remote...
Fetch complete.
✅ Repository is clean and in sync with 'origin/main'.
Monitoring changes in: '.'
Press Ctrl+C to exit.
-> Event received. Starting debounce timer...
-> Event received. Resetting debounce timer...

=======================================================
✅ DEBOUNCED ACTION! Processing 2 events...
=======================================================
[MODIFY] File content changed: ./notes.md
[MODIFY] File content changed: ./notes.md
-------------------------------------------------------
🚀 Executing git auto-commit...
-> Running: git add .
[SUCCESS] Staged changes.
-> Running: git commit -m "Update"
[SUCCESS] Committed changes:
[main a1b2c3d] Update
 1 file changed, 3 insertions(+)
-> Running: git push
[SUCCESS] Pushed changes.
-------------------------------------------------------
```

---

## 🚚 Releasing

Running `make tag` (or `make publish`) pushes a `vX.Y.Z` tag, which triggers [`.github/workflows/release.yml`](.github/workflows/release.yml). That workflow:

1. Creates a GitHub Release for the tag.
2. Cross-compiles `wac` for Linux (x86_64/aarch64, gnu/musl), macOS (x86_64/aarch64), and Windows (x86_64), and attaches archives + checksums.
3. Builds and attaches a `.deb` package via `cargo-deb`.
4. Pushes an updated formula to the `homebrew-tap` repo.
5. Opens a PR to `microsoft/winget-pkgs` bumping the winget manifest.

`cargo-binstall` needs no extra step — it resolves binaries directly from the release assets using `[package.metadata.binstall]` in `Cargo.toml`.

### One-time setup (already-configured channels need nothing further)

- **Homebrew**: create an empty `rodolfovillaruz/homebrew-tap` GitHub repo, then add a classic PAT with `repo` scope as the `HOMEBREW_TAP_TOKEN` secret on this repo.
- **winget**: fork `microsoft/winget-pkgs`, add a classic PAT with `public_repo` scope as the `WINGET_TOKEN` secret on this repo, and submit the **first** manifest manually (e.g. with [`komac`](https://github.com/russellbanks/Komac) or `wingetcreate`) — `winget-releaser` only automates version *updates* to an already-registered package.

## 📄 License

[MIT](LICENSE)

---

## 🙏 Acknowledgements

- [`notify`](https://crates.io/crates/notify) — cross-platform file system notifications.
- [`tokio`](https://tokio.rs/) — asynchronous runtime for Rust.
