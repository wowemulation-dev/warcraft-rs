# Pull Request

## Summary

<!-- Provide a brief description of the changes in this PR -->

## Type of Change

<!-- Check the type of change this PR introduces -->

- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing
  functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Test improvements
- [ ] Build system/dependency changes
- [ ] Security fix

## Changes Made

<!-- Describe the changes made in this PR -->

-
-
-

## Related Issues

<!-- Link to related issues -->

Fixes #(issue number)
Closes #(issue number)
Related to #(issue number)

## Testing

<!-- Describe how you tested your changes -->

### Test Cases Added/Modified

- [ ] Unit tests
- [ ] Integration tests
- [ ] Compliance tests (StormLib compatibility)
- [ ] Performance benchmarks
- [ ] Manual testing

### Test Results

<!-- Paste relevant test output -->

```bash
# Example test commands and results
cargo test
cargo test -p wow-mpq
cargo bench
```

### Tested On

<!-- Check all that apply -->

- [ ] Linux
- [ ] macOS
- [ ] Windows
- [ ] Cross-compilation targets

### WoW Versions Tested

<!-- Check versions you've tested with -->

- [ ] 1.12.1 (Vanilla)
- [ ] 2.4.3 (TBC)
- [ ] 3.3.5a (WotLK)
- [ ] 4.3.4 (Cataclysm)
- [ ] 5.4.8 (MoP)
- [ ] Other: _______________

## Quality Assurance

<!-- Confirm you've completed these steps -->

### Code Quality

- [ ] Code follows project style guidelines
- [ ] Self-review of code completed
- [ ] Code is properly documented
- [ ] No obvious performance regressions
- [ ] Error handling is appropriate

### Required Checks

- [ ] `cargo fmt --all` - Code is formatted
- [ ] `cargo clippy --all-targets --all-features` - No clippy warnings
- [ ] `cargo test --all-features` - All tests pass
- [ ] `cargo test --no-default-features` - Tests pass without features
- [ ] `cargo deny check` - No security/license issues
- [ ] Documentation builds successfully

### Compatibility

- [ ] No breaking changes to public API (or properly documented)
- [ ] Backward compatibility maintained where possible
- [ ] StormLib compatibility preserved (if applicable)
- [ ] Cross-platform compatibility verified

## Documentation

<!-- Check all that apply -->

- [ ] Updated relevant documentation in `docs/`
- [ ] Updated CHANGELOG.md
- [ ] Updated README.md (if applicable)
- [ ] Added/updated code examples
- [ ] Added/updated CLI help text
- [ ] API documentation updated (rustdoc)

## Benchmarks

<!-- If applicable, include benchmark results -->

### Performance Impact

- [ ] No performance impact
- [ ] Performance improvement (include metrics)
- [ ] Performance regression (justified and documented)

### Benchmark Results

<!-- Paste benchmark comparison if relevant -->

```
# Before:
test bench_parse_archive ... bench: 1,234 ns/iter (+/- 56)

# After:
test bench_parse_archive ... bench: 987 ns/iter (+/- 43)
```

## Breaking Changes

<!-- If this PR introduces breaking changes, describe them -->

### API Changes

<!-- List any changes to public APIs -->

### Migration Guide

<!-- Provide guidance for users to migrate their code -->

## Security Considerations

<!-- If applicable, describe security implications -->

- [ ] No security implications
- [ ] Security improvement
- [ ] Potential security impact (reviewed and justified)

## Additional Context

<!-- Add any other context about the PR -->

### Dependencies

<!-- List any new dependencies or dependency updates -->

### Known Limitations

<!-- Describe any known limitations or future work needed -->

### Screenshots/Examples

<!-- Include screenshots, command output, or examples if helpful -->

---

## Reviewer Notes

<!-- Notes for reviewers -->

### Areas of Focus

<!-- Highlight specific areas where you'd like reviewer attention -->

### Questions for Reviewers

<!-- Any specific questions you have for reviewers -->

---

**By submitting this PR, I confirm that:**

- [ ] I have read and agree to the [Contributing Guidelines](../CONTRIBUTING.md)
- [ ] This PR is ready for review (not a draft)
- [ ] I am willing to address feedback and make necessary changes
- [ ] I understand this may take time to review and merge