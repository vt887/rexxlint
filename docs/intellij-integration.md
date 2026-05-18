# rexxlint IntelliJ Integration Contract

This document specifies how an IntelliJ IDEA (or any JetBrains IDE) plugin should integrate with `rexxlint`.

## Integration Modes

### 1. Real-time Linting (External Linter)

The plugin should run `rexxlint check` on the file content. To avoid disk I/O and handle unsaved changes, use standard input.

**Command**:
```bash
rexxlint check --stdin --output json --path "$VIRTUAL_FILE_PATH"
```

**Input**: Send the current editor content to `stdin`.

**Output**: A JSON object containing an array of diagnostics.

**JSON Schema**:
```json
{
  "file": "stdin",
  "diagnostics": [
    {
      "code": "R001",
      "severity": "error",
      "message": "Missing required first-line Rexx comment",
      "span": {
        "start_line": 1,
        "start_col": 1,
        "end_line": 1,
        "end_col": 1
      },
      "fix": {
        "replacement": "/* header */",
        "span": { "start_line": 1, "start_col": 1, "end_line": 1, "end_col": 1 }
      }
    }
  ]
}
```

### 2. Formatting

The plugin should implement the `FormattingService` or `AsyncDocumentFormattingService`.

**Command**:
```bash
rexxlint format --stdin --path "$VIRTUAL_FILE_PATH"
```

**Input**: Send the current editor content to `stdin`.

**`--path`**: Pass the editor's virtual file path so that diagnostic spans and diff headers reference the correct filename. If omitted, defaults to `<stdin>`.

**Output**: The formatted source code on `stdout`.

**Error Handling**: If `stderr` is not empty or exit code is 2, do not replace editor content.

### 3. Save-on-Check (Optional)

To verify if a file is already formatted without changing it:

**Command**:
```bash
rexxlint format --check <path>
```

**Exit Code**: 0 if formatted, 1 if changes are needed.

## Exit Codes

- `0`: Success. No diagnostics found (in `check`) or file is formatted (in `format --check`).
- `1`: Diagnostics found or formatting needed.
- `2`: Fatal error (Configuration error, IO error, Internal crash).

## Configuration

The plugin should allow the user to configure:
1.  **Path to `rexxlint` executable**: Default to `rexxlint` in PATH.
2.  **Optional config file**: If not provided, `rexxlint` will discover `rexxlint.toml` automatically.

## Performance Recommendations

- **Timeout**: Kill the process if it takes longer than 2 seconds.
- **Debouncing**: For real-time linting, wait 300-500ms after the last keystroke before triggering the check.
- **Concurrency**: Only one `rexxlint` instance per file should be running at a time. Cancel previous processes if a new one is triggered.

## Diagnostics Handling

- Use `span.start_line` and `span.start_col` for the primary gutter icon and highlighting start.
- If `fix` is present, it can be surfaced as an "Intention Action" (Alt+Enter).
