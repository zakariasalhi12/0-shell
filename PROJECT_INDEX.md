# 0-Shell Project Index

## Overview
0-Shell is a Unix-like shell implementation written in Rust. It provides an interactive command-line interface with support for basic shell features including command execution, history, environment variables, and various built-in commands.

## Project Structure

### Core Architecture
```
0-shell/
├── Cargo.toml              # Rust project configuration and dependencies
├── Dockerfile              # Container configuration for development/testing
├── Makefile                # Build and run commands
├── README.md               # Project documentation
└── src/                    # Source code directory
```

### Dependencies
- **colored**: Terminal color output
- **dirs**: Cross-platform directory utilities
- **fs**: File system operations
- **lazy_static**: Static initialization
- **regex**: Regular expressions
- **chrono**: Date/time handling
- **users**: User/group information
- **termion**: Terminal input/output handling
- **whoami**: Username utilities

## Source Code Architecture

### Main Entry Points
- **`src/main.rs`**: Application entry point, initializes and runs the shell
- **`src/lib.rs`**: Library root, exports public modules and utilities

### Core Components

#### 1. Shell Interface (`src/events_handler.rs`)
- **Shell struct**: Main shell state management
  - Terminal I/O handling (stdin/stdout)
  - Command buffer management
  - Cursor position tracking
  - History integration
  - Environment state

**Key Features:**
- Raw terminal mode support
- Interactive command editing
- History navigation (Up/Down arrows)
- Cursor movement (Left/Right arrows)
- Terminal clearing (Ctrl+L)
- Signal handling (Ctrl+C, Ctrl+D, Ctrl+Z)

#### 2. Lexical Analysis (`src/lexer/`)
- **`types.rs`**: Token and word definitions
  - `Token`: Lexical tokens (Word, Pipe, Redirect, etc.)
  - `Word`: Shell words with parts and quoting
  - `WordPart`: Word components (Literal, VariableSubstitution, etc.)
  - `QuoteType`: Quoting types (Single, Double, None)
  - `State`: Lexer state machine states

- **`tokenize.rs`**: Tokenizer implementation
  - Character-by-character parsing
  - Variable substitution recognition (`$VAR`, `${VAR}`)
  - Arithmetic substitution (`$((expr))`)
  - Command substitution (`$(cmd)`)
  - Quote handling
  - Redirection operators

#### 3. Parsing (`src/parser/`)
- **`types.rs`**: Abstract Syntax Tree (AST) definitions
  - `AstNode`: All shell constructs (Command, Pipeline, If, While, etc.)
  - `ArithmeticExpr`: Mathematical expressions
  - `Redirect`: I/O redirection specifications
  - `BinaryOperator`/`UnaryOperator`: Arithmetic operators

**Parser Modules:**
- **`parse_command.rs`**: Command parsing with assignments and redirections
- **`parse_pipeline.rs`**: Pipeline operator (`|`) parsing
- **`parse_sequence.rs`**: Command sequences (`;`)
- **`parse_if.rs`**: Conditional statements
- **`parse_while.rs`**: Loop constructs
- **`parse_for.rs`**: For loops
- **`parse_function.rs`**: Function definitions
- **`parse_group.rs`**: Command grouping `{...}`
- **`parse_redirection.rs`**: I/O redirection parsing
- **`parse_assignment.rs`**: Variable assignments

#### 4. Execution Engine (`src/exec.rs`)
- **`execute()`**: Main execution function
  - AST traversal and interpretation
  - Built-in command dispatch
  - External command execution
  - Environment variable expansion
  - Error handling and exit codes

- **`build_command()`**: Command factory
  - Maps command names to implementations
  - Supports: echo, cd, ls, pwd, cat, cp, rm, mv, mkdir, export, exit

#### 5. Environment Management (`src/envirement.rs`)
- **ShellEnv struct**: Complete shell environment
  - Shell variables (`HashMap<String, String>`)
  - Arithmetic variables (`HashMap<String, i64>`)
  - User-defined functions (`HashMap<String, AstNode>`)
  - Job control (`HashMap<usize, Job>`)
  - Exit status tracking
  - Shell start time

**Key Methods:**
- `set_var()` / `get_var()`: Variable management
- `set_arith()` / `get_arith()`: Arithmetic variables
- `add_job()` / `get_job()`: Job control
- `set_last_status()` / `get_last_status()`: Exit status

#### 6. Built-in Commands (`src/commands/`)
Each command implements the `ShellCommand` trait:

- **`echo.rs`**: Print text with quote handling
- **`cd.rs`**: Change directory
- **`ls.rs`**: List directory contents with formatting options
- **`pwd.rs`**: Print working directory
- **`cat.rs`**: Concatenate and display files
- **`cp.rs`**: Copy files and directories
- **`rm.rs`**: Remove files and directories
- **`mv.rs`**: Move/rename files
- **`mkdir.rs`**: Create directories
- **`export.rs`**: Set environment variables

#### 7. Features

**History System (`src/features/history.rs`)**
- Persistent command history storage
- Navigation through history (Up/Down arrows)
- Automatic history saving
- File-based persistence (`~/.0-shell_history`)

**Job Control (`src/jobs.rs`)**
- Background job management
- Job status tracking (Running, Stopped, Done)
- Process group management
- Job ID assignment

#### 8. Utilities

**Error Handling (`src/error.rs`)**
- `ShellError` enum with comprehensive error types
- IO, syntax, parsing, evaluation, execution errors
- Error conversion traits

**Configuration (`src/config.rs`)**
- Environment variable initialization
- User information setup
- Shell configuration defaults

**Expansion (`src/expansion.rs`)**
- Variable expansion (`$VAR`)
- Default value expansion (`${VAR:-default}`)
- Arithmetic expansion (`$((expr))`)

**Evaluation (`src/eval.rs`)**
- Arithmetic expression evaluation
- Mathematical operations
- Variable substitution in expressions

## Command Execution Flow

1. **Input**: User types command in interactive shell
2. **Lexical Analysis**: `Tokenizer` converts input to tokens
3. **Parsing**: `Parser` builds AST from tokens
4. **Expansion**: Variables and expressions are expanded
5. **Execution**: `execute()` interprets AST and runs commands
6. **Output**: Results displayed, history updated

## Supported Shell Features

### Basic Commands
- File operations: ls, cat, cp, rm, mv, mkdir
- Navigation: cd, pwd
- Output: echo
- Environment: export

### Shell Constructs
- Command sequences (`;`)
- Pipelines (`|`)
- Logical operators (`&&`, `||`)
- Background execution (`&`)
- Variable assignments
- I/O redirection (`>`, `<`, `>>`)

### Interactive Features
- Command history
- Line editing
- Cursor movement
- Terminal control
- Signal handling

### Planned Features (TODO)
- Control structures (if, while, for, case)
- Function definitions
- Subshells
- Job control
- Arithmetic expressions
- Advanced expansion

## Development Setup

### Local Development
```bash
cargo build
cargo run
```

### Docker Development
```bash
make build
make run
```

### Testing
```bash
cargo test
```

## Architecture Patterns

### Trait-based Design
- `ShellCommand` trait for command implementations
- Consistent interface across all built-ins

### State Machine
- Lexer uses state machine for token recognition
- Parser maintains position and lookahead

### Environment Passing
- `ShellEnv` passed through execution chain
- Mutable environment updates

### Error Propagation
- `Result<T, ShellError>` throughout codebase
- Comprehensive error handling

## Code Quality

### Rust Features Used
- Pattern matching extensively
- Error handling with `Result`
- Generic types and traits
- Memory safety without garbage collection
- Zero-cost abstractions

### Performance Considerations
- Efficient string handling
- Minimal allocations
- Direct terminal I/O
- Lazy evaluation where appropriate

## Future Enhancements

### Planned Features
- Complete control structure implementation
- Advanced job control
- Signal handling
- Subshell support
- Function definitions
- Arithmetic expressions
- Advanced expansion features

### Potential Improvements
- Better error messages
- More comprehensive testing
- Performance optimizations
- Additional built-in commands
- Plugin system
- Configuration file support 