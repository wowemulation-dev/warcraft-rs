# Security Policy

## Supported Versions

The following versions of `warcraft-rs` are currently being supported with
security updates:

| Version | Supported             |
| ------- | --------------------- |
| 0.1.x   | :white_check_mark: |
| < 0.1   | :x:                |

## Reporting a Vulnerability

We take the security of `warcraft-rs` seriously. If you believe you have found a
security vulnerability, please report it to us as described below.

### Please DO NOT

- Open a public issue on GitHub
- Discuss the vulnerability in public forums
- Exploit the vulnerability beyond what is necessary to demonstrate it

### Please DO

1. **Email us directly** at: <daniel@kogito.network>
   - Use the subject line: `[SECURITY] warcraft-rs - Brief Description`
   - Include "warcraft-rs Security" in the subject line

2. **Include the following information:**
   - Type of vulnerability (e.g., buffer overflow, path traversal, etc.)
   - Affected component(s) and version(s)
   - Step-by-step instructions to reproduce the issue
   - Proof-of-concept or exploit code (if possible)
   - Impact assessment and potential attack scenarios
   - Any suggested fixes or mitigations

3. **Use GPG encryption** (optional but recommended):
   - Our GPG key can be requested via email
   - This ensures confidential communication

## Response Timeline

- **Initial Response**: Within 48 hours, we will acknowledge receipt of your
  report
- **Assessment**: Within 7 days, we will provide an initial assessment and
  expected timeline
- **Resolution**: We aim to resolve critical issues within 30 days, depending on
  complexity

## What to Expect

1. **Acknowledgment**: You'll receive confirmation that we've received your
   report
2. **Communication**: We'll keep you informed about our progress
3. **Credit**: With your permission, we'll acknowledge your contribution when
   the issue is resolved
4. **Disclosure**: We'll work with you to establish an appropriate disclosure
   timeline

## Security Considerations for data files

While `warcraft-rs` handles data files from World of Warcraft, security
considerations include:

### Potential Security Risks

1. **Malformed Files**: Crafted files could potentially cause:
   - Buffer overflows
   - Excessive memory allocation
   - Infinite loops
   - Integer overflows

2. **Path Traversal**: When loading listfiles or files:
   - Validate all file paths
   - Prevent directory traversal attacks

3. **Resource Exhaustion**: Large or crafted files could cause:
   - Memory exhaustion
   - CPU exhaustion through algorithmic complexity

### Our Security Measures

- Input validation on all file operations
- Bounds checking on all array accesses
- Safe string handling using Rust's memory safety
- Limited recursion depth for complex operations
- Resource limits for memory allocation

## Bug Bounty

Currently, we do not offer a bug bounty program. However, we greatly appreciate
security researchers who responsibly disclose vulnerabilities and will
acknowledge their contributions.

## Security Updates

Security updates will be released as patch versions (e.g., 0.1.1, 0.1.2) and
announced through:

- GitHub Security Advisories
- Release notes
- The CHANGELOG.md file

## Additional Resources

- [Rust Security Guidelines](https://rustsec.org/)

---

Thank you for helping keep `warcraft-rs` and its users safe!
