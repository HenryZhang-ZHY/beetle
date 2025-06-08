# Debugging E2E Test Failures

## Summary

This document captures the debugging experience and resolution for a failing end-to-end test in the beetle CLI application.

## Problem Description

The e2e test `test_c_programmer_onboarding` was failing with the following error:

```
test test_c_programmer_onboarding ... FAILED
thread 'test_c_programmer_onboarding' panicked at apps/cli/tests/e2e_test.rs:167:10:
Search for 'int main' should succeed: Error("expected value", line: 1, column: 1)
```

## Root Cause Analysis

The issue was caused by **ANSI color codes in CLI output** interfering with JSON parsing. Specifically:

1. **Logging interference**: The CLI was outputting colored log messages from the `tracing` framework
2. **JSON parsing failure**: The `serde_json::from_str()` function was trying to parse ANSI escape codes as JSON, causing it to fail at line 1, column 1
3. **Environment variable**: The test environment wasn't configured to suppress log output

## Debug Output Example

The raw CLI output contained:
```
[2m2025-07-19T04:05:28.563426Z[0m [32m INFO[0m ...
{
  "payload": [...],
  "status": "success"
}
```

## Solution

The fix involved setting the `RUST_LOG=error` environment variable for all CLI commands in the test to suppress info-level logging:

```rust
let search_output = Command::cargo_bin("beetle")?
    .args(["search", "-i", index_name, "-q", query, "--format", "json"])
    .env("RUST_LOG", "error")  // Disable info logs
    .output()?;
```

## Files Modified

- `apps/cli/tests/e2e_test.rs`: Added `.env("RUST_LOG", "error")` to all CLI command invocations:
  - `execute_search()` function
  - `beetle new` command
  - `beetle update` commands (both instances)

## Key Learnings

1. **Logging in tests**: CLI applications with logging frameworks can interfere with test output parsing
2. **ANSI codes**: Color codes in terminal output can break JSON parsing
3. **Environment control**: Use environment variables to control logging levels in test environments
4. **Debug techniques**: Adding debug output to see raw CLI responses helps identify parsing issues

## Testing the Fix

After applying the fix, the test passes:

```
running 1 test
test test_c_programmer_onboarding ... ok

test result: ok. 1 passed; 0 failed
```

## Future Recommendations

1. **Standardize test setup**: Consider creating a test helper that automatically sets appropriate environment variables
2. **Output format validation**: Add validation that CLI output matches expected format before parsing
3. **Logging configuration**: Document logging level requirements for different environments
4. **CI considerations**: Ensure CI environments handle logging consistently across platforms