# Plan: Add ripgrep-all Document Search Support

## Overview

Add support for searching through binary documents (PDFs, Word docs, etc.) using ripgrep-all (rga). This will be implemented through two complementary approaches:

1. **Bash tool enhancement**: Add `rga` to the allowed bash commands list for advanced users
2. **Dedicated search_docs tool**: Create a specialized tool with intelligent defaults for document searching

## Implementation Steps

### Step 1: Add `rga` to Bash Allowed Commands

**File**: `src/tools/bash.rs`

**Change**: Line 16 - Update `ALLOWED_COMMANDS` constant

```rust
const ALLOWED_COMMANDS: &[&str] = &[
    "grep", "find", "cat", "head", "tail", "ls", "tree", "wc", "file", "rg", "rga",
];
```

**Rationale**: Allows models trained on rga usage to leverage it directly in bash commands for complex pipelines.

### Step 2: Create Dedicated `search_docs` Tool

**File**: `src/tools/search_docs.rs` (new file)

**Implementation Details**:

#### 2.1 Constants
- `TIMEOUT_SECS: u64 = 30` - Same as bash tool
- `MAX_COUNT: usize = 100` - Limit results to prevent overwhelming output
- `CONTEXT_LINES: usize = 2` - Show 2 lines before/after matches

#### 2.2 Args Struct
```rust
#[derive(Deserialize)]
pub struct SearchDocsArgs {
    /// The search query/pattern to find in documents
    pub query: String,
    
    /// Optional path or glob pattern to search in (defaults to current directory)
    pub path: Option<String>,
}
```

#### 2.3 Error Enum
```rust
#[derive(Debug, Error)]
pub enum SearchDocsError {
    #[error("rga command not found. Please install ripgrep-all: https://github.com/phiresky/ripgrep-all")]
    RgaNotInstalled,
    
    #[error("Search query is empty")]
    EmptyQuery,
    
    #[error("Search timed out after {0} seconds")]
    Timeout(u64),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Search failed with exit code {0}: {1}")]
    SearchFailed(i32, String),
}
```

#### 2.4 Tool Struct
```rust
#[derive(Deserialize, Serialize)]
pub struct SearchDocs {
    #[serde(skip)]
    base_dir: PathBuf,
}

impl SearchDocs {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }
}
```

#### 2.5 Tool Trait Implementation

**Definition method**:
- Name: `"search_docs"`
- Description: "Search through documents (PDFs, Word docs, Excel, etc.) using ripgrep-all. Automatically handles binary formats and extracts text. Use this when you need to find content in non-text files."
- Parameters: JSON schema for `query` (required) and `path` (optional)

**Call method**:
1. Validate query is not empty
2. Build rga command with flags:
   - `-i` - case-insensitive search
   - `--max-count {MAX_COUNT}` - limit results per file
   - `--context {CONTEXT_LINES}` - show context around matches
   - `--color never` - no ANSI color codes
   - `{query}` - the search pattern
   - `{path or "."}` - search location
3. Execute via `tokio::process::Command` in `base_dir`
4. Apply 30-second timeout
5. Handle results:
   - Success (exit 0): return stdout
   - Not found (exit 1): return "No matches found"
   - rga not installed (exit 127): return `RgaNotInstalled` error
   - Other errors: return `SearchFailed` with stderr
   - Timeout: return `Timeout` error

#### 2.6 Key Features
- **Smart defaults**: Preconfigured with reasonable flags
- **Binary format support**: Searches PDFs, DOCX, XLSX, PPTX, ODT, EPUB, etc.
- **Timeout protection**: Won't hang on massive document sets
- **Result limiting**: Prevents overwhelming output
- **Context preservation**: Shows surrounding lines for better understanding
- **Natural error handling**: Returns appropriate errors when rga is missing or fails

### Step 3: Register Tool in Module System

**File**: `src/tools.rs`

**Changes**:

1. Add module declaration (after `mod read_file;`):
```rust
mod search_docs;
```

2. Add public re-export (after `pub use read_file::{...};`):
```rust
pub use search_docs::{SearchDocs, SearchDocsArgs};
```

3. Add to `ToolType` enum (in alphabetical order):
```rust
pub enum ToolType {
    Bash(BashCommandArgs),
    ReadFile(ReadFileArgs),
    SearchDocs(SearchDocsArgs),
}
```

4. Update `ToolType::as_string()` method:
```rust
pub fn as_string(&self) -> String {
    match self {
        ToolType::Bash(args) => {
            format!("bash(command: {})", args.command)
        }
        ToolType::ReadFile(args) => {
            format!("read_file(path: {})", args.path)
        }
        ToolType::SearchDocs(args) => {
            let path = args.path.as_ref().map(|p| p.as_str()).unwrap_or(".");
            format!("search_docs(query: {}, path: {})", args.query, path)
        }
    }
}
```

### Step 4: Initialize Tool in Main

**File**: `src/main.rs`

**Change**: Add to agent builder chain (after `.tool(BashCommand::new(...))`):

```rust
let agent = AgentBuilder::new(model)
    .preamble(&preamble)
    .default_max_turns(args.max_turns)
    .tool(ReadFile::new(base_dir.clone()))
    .tool(BashCommand::new(base_dir.clone()))
    .tool(SearchDocs::new(base_dir.clone()))
    .build();
```

## Testing Strategy

### Manual Testing

1. **Verify rga in bash**:
   ```bash
   # Should work if rga is installed
   bash: rga "search term" docs/
   ```

2. **Test search_docs tool**:
   ```bash
   # Search all documents
   search_docs: query="important keyword"
   
   # Search specific path
   search_docs: query="contract terms", path="legal/*.pdf"
   
   # Verify error when rga not installed (in clean environment)
   search_docs: query="test"
   # Should return clear error message
   ```

3. **Test edge cases**:
   - Empty query → Should return EmptyQuery error
   - Non-existent path → Should return "No matches found"
   - Very large document set → Should timeout gracefully at 30s
   - Binary files without text → Should handle gracefully

### Unit Tests (Optional Enhancement)

Add to `src/tools/search_docs.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty_query() {
        // Test that empty queries are rejected
    }

    #[test]
    fn test_command_construction() {
        // Test that rga command is built correctly with flags
    }
}
```

## Dependencies

### System Requirements
- **ripgrep-all** must be installed on the system
- Installation varies by OS:
  - macOS: `brew install ripgrep-all`
  - Linux: `cargo install ripgrep-all` or package manager
  - Windows: Via cargo or scoop

### Rust Dependencies
No new Cargo dependencies needed - reuses existing:
- `tokio` - async command execution
- `serde` - args deserialization
- `thiserror` - error types
- `rig` - tool trait

## User-Facing Changes

### New Capabilities

1. **Bash tool**: Can now use `rga` command directly
   - Example: `rga --type pdf "quarterly report" | head -20`

2. **search_docs tool**: Dedicated document search
   - Optimized for common use cases
   - Clear error messages
   - Automatic handling of various document formats

### Documentation Updates Needed

Update README.md or add TOOLS.md with:

```markdown
## search_docs

Search through binary documents (PDFs, Word, Excel, etc.) using ripgrep-all.

**Arguments:**
- `query` (required): Search pattern/text to find
- `path` (optional): File path or glob pattern (default: current directory)

**Examples:**
- `search_docs(query: "contract terms")`
- `search_docs(query: "Q[1-4] 2024", path: "reports/*.pdf")`

**Supported formats:** PDF, DOCX, XLSX, PPTX, ODT, EPUB, and more

**Requirements:** Requires ripgrep-all to be installed
```

## Rollout Checklist

- [ ] Step 1: Add `rga` to bash allowed commands
- [ ] Step 2: Implement `search_docs.rs` with all features
- [ ] Step 3: Register in `tools.rs` module system
- [ ] Step 4: Initialize in `main.rs` agent builder
- [ ] Run `cargo make test` to verify compilation
- [ ] Run `cargo make check-all` for linting
- [ ] Manual testing with sample documents
- [ ] Update documentation

## Future Enhancements (Out of Scope)

1. **File type filters**: Add `file_types: Vec<String>` arg to restrict search
2. **Preview option**: Add flag to show snippets from each matching file
3. **Statistics**: Return match counts, file counts, search duration
4. **Streaming results**: For very large document sets, stream results progressively
5. **Cache support**: Cache document text extraction for repeated searches
6. **Advanced options**: Expose more rga flags (regex mode, encoding, etc.)

## Success Criteria

✅ Agent can search PDFs and other documents via both tools
✅ Graceful error handling when rga is not installed
✅ No hangs on large document sets (timeout protection)
✅ Results are limited and contextual (not overwhelming)
✅ Code follows established patterns (matches bash.rs and read_file.rs)
✅ All tests pass (`cargo make test` and `cargo make check-all`)
