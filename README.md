# Rusty Commit Saver

**A simple Rust tool to automatically log your Git commits into Markdown files for Obsidian.**

Rusty Commit Saver captures each commit’s:

- **Timestamp**
- **Commit message**
- **Repository URL**
- **Branch name**
- **Commit hash**

and appends it to a dated diary entry in your Wiki directory.

---

## 🚀 Features

- Automatic diary entry creation with YAML frontmatter and table header
- Timestamped commit rows formatted for Obsidian
- Customizable storage path under `📅 Diaries/0. Commits/YYYY/MM-MMMM/`
- Preconfigured hooks (via Nix + pre-commit) to ensure code quality

---

## 📦 Installation

1. Clone the repository

   ```bash
   git clone https://github.com/your-username/rusty-commit-saver.git
   cd rusty-commit-saver
   ```

2. Enter the Nix development shell

   ```bash
   devenv shell
   ```

3. Build the project

   ```bash
   cargo build --release
   ```

4. (Optional) Install the binary to your PATH

   ```bash
   cargo install --path .
   ```

---

## ⚙️ Usage

Simply commit as usual. The pre-commit hook will:

1. Run linters (clippy, rustfmt, etc.) inside the Nix shell
2. Invoke Rusty Commit Saver to log the commit

If you prefer manual invocation:

```bash
rusty-commit-saver
```

Your commit will be appended to:

```text
~/Documents/Wiki/📅 Diaries/0. Commits/YYYY/MM-MMMM/YYYY-MM-DD.md
```

---

## 🛠 Configuration

- **`rust-toolchain.toml`** pins Rust 1.89.0
- **`devenv.nix`** provisions Rust, Clippy, rustfmt, and Git hooks
- **`.pre-commit-config.yaml`** defines all pre-commit checks
- **`treefmt.toml`** configures `treefmt` and formatters

---

## 📈 Roadmap & Improvements

There are **many enhancements** planned:

- Support for configurable output formats (CSV, JSON)
- Parallel commit logging across multiple repos
- Interactive CLI flags and richer metadata (author, files changed)
- Integration with other note-taking systems (Logseq, Joplin)
- Improved error handling and user feedback
- Unit tests and CI pipeline for automated releases

Contributions welcome! Feel free to open issues or submit PRs.

---

## 💖 Contributing

1. Fork the repo
2. Create a feature branch
3. Write tests and update `README.md`
4. Submit a pull request

---

## 📄 License

MIT © 2025 Chess7th
