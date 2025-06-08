# Contributing to `warcraft-rs`

First off, thank you for considering contributing to `warcraft-rs`! 🎉 We're
excited to have you join our community. This document provides guidelines and
instructions to make the contribution process smooth and effective for everyone.

## 📝 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
  - [Set Up Your Environment](#set-up-your-environment)
  - [Find Issues to Work On](#find-issues-to-work-on)
- [Making Contributions](#making-contributions)
  - [Create a Branch](#create-a-branch)
  - [Make Your Changes](#make-your-changes)
  - [Test Your Changes](#test-your-changes)
  - [Submit a Pull Request](#submit-a-pull-request)
- [Coding Guidelines](#coding-guidelines)
- [Documentation](#documentation)
- [Community](#community)
- [Recognition](#recognition)

## Code of Conduct

This project and everyone participating in it is governed by our
[Code of Conduct](CODE_OF_CONDUCT.md). By participating, you are expected to
uphold this code. Please report unacceptable behavior to
[me](mailto:daniel@kogito.network).

## Getting Started

### Set Up Your Environment

1. **Fork the repository**: Click the "Fork" button at the top right of the
   [repository page](https://github.com/wowemulation-dev/warcraft-rs).

2. **Clone your fork**:

   ```bash
   git clone https://github.com/your-username/warcraft-rs.git
   cd warcraft-rs
   ```

3. **Set up the upstream remote**:

   ```bash
   git remote add upstream https://github.com/wowemulation-dev/warcraft-rs.git
   ```

4. **Install Rust and dependencies**:
   - Install [Rust](https://www.rust-lang.org/tools/install) if you haven't already
   - Run `cargo build` to download dependencies and build the project

### Find Issues to Work On

- Check the [Issues](https://github.com/wowemulation-dev/warcraft-rs/issues)
  tab for tasks labeled "good first issue" or "help wanted"
- The [TODO.md](TODO.md) file contains a list of planned features and improvements
- Feel free to ask in the [Discussions](https://github.com/wowemulation-dev/warcraft-rs/discussions)
  if you need help finding a suitable task

## Making Contributions

### Create a Branch

```bash
# Make sure you're up to date
git checkout main
git pull upstream main

# Create a new branch
git checkout -b my-feature-branch
```

Name your branch descriptively, e.g., `add-blp-support` or `fix-header-reading`.

### Make Your Changes

1. **Code**: Implement your changes following our [Coding Guidelines](#coding-guidelines)
2. **Tests**: Add or update tests for your changes
3. **Documentation**: Update documentation as needed

### Test Your Changes

Before submitting your changes, make sure to run:

```bash
# Format your code
cargo fmt

# Check for common issues
cargo clippy --all-targets --all-features

# Run tests
cargo test

# Check dependencies for security issues
cargo deny check
```

If you've added a new feature, consider adding a benchmark:

```bash
cargo bench
```

#### Continuous Integration

Our CI/CD pipeline automatically runs the following checks on all pull requests:

1. **Quick Checks** (runs first, fails fast):
   - Code formatting (`cargo fmt`)
   - Compilation check (`cargo check`)
   - Linting (`cargo clippy`)

2. **Test Matrix**:
   - Runs on Linux, Windows, and macOS
   - Tests with Rust stable, beta, and MSRV (1.86.0)
   - Tests with all features and no default features

3. **Documentation**:
   - Builds documentation with warnings as errors
   - Checks for broken documentation links

4. **Code Coverage**:
   - Measures test coverage with cargo-llvm-cov
   - Reports to Codecov

5. **Security Audits** (runs on schedule):
   - Dependency vulnerability scanning
   - License compliance checks

6. **Cross-Platform Builds**:
   - Builds for multiple targets including ARM64 and Windows
   - Uses cross-compilation where needed

7. **Benchmarks** (on PRs):
   - Compares performance against the base branch
   - Automatically comments results on PRs

8. **Release Pipeline**:
   - Automated releases on version tags
   - Builds binaries for all supported platforms
   - Publishes crates to crates.io
   - Creates GitHub releases with artifacts

### Submit a Pull Request

1. **Push your changes**:

   ```bash
   git push origin my-feature-branch
   ```

2. **Create a Pull Request**: Go to the [repository page](https://github.com/wowemulation-dev/warcraft-rs) and click "New Pull Request"

3. **Describe your changes**:
   - Provide a clear title
   - Explain what you've changed and why
   - Reference any related issues (e.g., "Fixes #42")
   - Include any special instructions for testing

4. **Respond to feedback**: Maintainers may suggest changes to your PR. Discuss
   and make any necessary updates.

## Coding Guidelines

- Follow Rust's official [style guide](https://doc.rust-lang.org/1.0.0/style/README.html)
- Use meaningful variable and function names
- Write clear comments for complex logic
- Keep functions focused on a single responsibility
- Add proper error handling using `Result` and `Option` types
- Include unit tests for all new functionality

## Documentation

Good documentation is essential for our project:

- **Code Comments**: Document functions and complex logic
- **Examples**: Add examples for new features in the `examples/` directory
- **README**: Update the README if your changes add new features or change existing functionality
- **Rustdoc**: Add documentation comments (`///`) to public API elements

## Community

We're building a friendly and inclusive community:

- **Ask questions**: If you're unsure about something, ask in [Discussions](https://github.com/wowemulation-dev/warcraft-rs/discussions)
- **Help others**: Share your knowledge by answering questions
- **Be respectful**: Always be kind and constructive in communications

## Recognition

We value all contributions, big and small! Contributors will be:

- Listed in our [CONTRIBUTORS.md](CONTRIBUTORS.md) file
- Mentioned in release notes when their contributions are included
- Recognized in project documentation

---

## 🚀 Beginner's Guide to Open Source Contribution

New to open source or Rust? Here are some tips to get started:

### Understanding Git & GitHub

1. **Fork**: Creates your personal copy of the repository
2. **Clone**: Downloads the repository to your computer
3. **Branch**: Creates a separate workspace for your changes
4. **Commit**: Saves your changes with a message explaining what you did
5. **Push**: Uploads your changes to GitHub
6. **Pull Request**: Asks the project maintainers to review and merge your changes

### First-Time Contributor Tips

1. **Start small**: Fix a typo, improve documentation, or tackle a "good first issue"
2. **Ask questions**: Don't hesitate to ask for clarification or help
3. **Be patient**: Maintainers are often busy; give them time to respond
4. **Stay positive**: Be open to feedback and suggestions

### Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Git Tutorial](https://www.atlassian.com/git/tutorials)
- [How to Contribute to Open Source](https://opensource.guide/how-to-contribute/)
- [Understanding the GitHub Flow](https://guides.github.com/introduction/flow/)

---

Thank you for contributing to `warcraft-rs`! Your efforts help make this project
better for everyone. If you have any questions or need assistance, please reach
out to us in the Discussions section.
