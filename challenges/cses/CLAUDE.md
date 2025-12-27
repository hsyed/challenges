# CSES Task System Guide

This documents the CSES implementation test harness.

## Architecture Overview

The task system uses a `TaskGroup` abstraction for automatic task registration and discovery:

- **TaskGroup** (`lib.rs`): Manages task registration, lookup, and execution
- **Task modules**: Individual problem implementations (e.g., `digit_queries.rs`)
- **Category modules**: Group related tasks (e.g., `introductory/mod.rs`)
- **Main dispatch** (`main.rs`): Minimal one-line dispatch to category TaskGroups

## Adding a New Task to Existing Category

To add a new task to the `introductory` category:

### 1. Create the task file

Create `src/introductory/your_task_name.rs`:

```rust
//https://cses.fi/problemset/task/YOUR_TASK_ID
use crate::{Scanner, Writer};

pub fn solve(input: &mut Scanner, out: &mut Writer) {
    // Read input
    let n: usize = input.parse();

    // Solve the problem
    let result = compute(n);

    // Write output
    out.println(result);
}

fn compute(n: usize) -> usize {
    // Your implementation
    n * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        assert_eq!(compute(5), 10);
    }
}
```

**Required signature**: `pub fn solve(input: &mut Scanner, out: &mut Writer)`

### 2. Register in mod.rs

Add two lines to `src/introductory/mod.rs`:

```rust
pub mod your_task_name;  // Declare the module

pub fn tasks() -> TaskGroup {
    TaskGroup::new("introductory")
        .add("digit_queries", digit_queries::solve)
        .add("grid_path_description", grid_path_description::solve)
        .add("your_task_name", your_task_name::solve)  // Register the task
}
```

**That's it!** The task is now available via CLI:

```bash
cargo run -- introductory your_task_name
```

### 3. Add test cases (optional)

Create test files in `data/introductory/your_task_name/`:

- `1.in` - First test input
- `1.out` - Expected output
- `2.in`, `2.out`, etc.

The testing framework automatically discovers and runs all numbered test cases.

## Creating a New Task Group (Category)

To add a new category like "sorting" or "graph":

### 1. Create the category module

Create `src/sorting/mod.rs`:

```rust
// Sorting algorithms category
// Add problem modules here as they are implemented

pub mod bubble_sort;
pub mod merge_sort;

use crate::TaskGroup;

pub fn tasks() -> TaskGroup {
    TaskGroup::new("sorting")
        .add("bubble_sort", bubble_sort::solve)
        .add("merge_sort", merge_sort::solve)
}
```

### 2. Declare module in lib.rs

Add to `src/lib.rs`:

```rust
pub mod introductory;
pub mod sorting;  // Add this line
```

### 3. Add CLI subcommand

Update `src/main.rs`:

**Add to the Commands enum:**

```rust
#[derive(Subcommand)]
enum Commands {
    /// Introductory Problems
    Introductory {
        problem: String,
    },
    /// Sorting Problems
    Sorting {
        problem: String,
    },
}
```

**Add to the match statement:**

```rust
match cli.command {
    Commands::Introductory { problem } => {
        cses::introductory::tasks().run(&problem);
    }
    Commands::Sorting { problem } => {
        cses::sorting::tasks().run(&problem);
    }
}
```

### 4. Create task files

- Add individual tasks as described in "Adding a New Task" above.
- Do not implement beyond basic parsing if you know the structure of the task

## Task Structure Convention

Each task file should follow this structure:

```rust
//https://cses.fi/problemset/task/TASK_ID
use crate::{Scanner, Writer};

// Main entry point - required signature
pub fn solve(input: &mut Scanner, out: &mut Writer) {
    // Implementation
}

// Helper functions (private)
fn helper(x: usize) -> usize {
    // ...
}

// Unit tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_helper() {
        assert_eq!(helper(5), expected);
    }
}
```

## Scanner and Writer API

### Scanner methods

- `input.parse::<T>()` - Parse next line as type T (inferred or explicit)
- `input.next_line()` - Read entire line as String

### Writer methods

- `out.println(value)` - Write value with newline
- `out.print(value)` - Write value without newline
- `writeln!(out, "{}", value)` - Format and write with newline (std macro)

## Testing

### Run specific task

```bash
cargo run -- introductory digit_queries
```

### Run with invalid task name (lists available)

```bash
cargo run -- introductory invalid
# Output:
# Unknown problem: invalid
#
# Available problems in 'introductory':
#   digit_queries
#   grid_path_description
```

### Integration tests

Place `.in` and `.out` files in `data/{category}/{task_name}/` and the framework automatically runs them.

## Benefits of This System

1. **No main.rs updates**: Adding tasks only touches category mod.rs
2. **Automatic help**: Unknown tasks list all available tasks
3. **Type safety**: Function pointers ensure correct signatures
4. **Scalable**: Easy to add categories with same pattern
5. **Simple API**: Clear, readable registration with `.add()`

## Key Files

- `src/lib.rs` - TaskGroup definition and testing utilities
- `src/main.rs` - CLI dispatch (minimal boilerplate)
- `src/{category}/mod.rs` - Task registration for each category
- `src/{category}/{task}.rs` - Individual task implementations
