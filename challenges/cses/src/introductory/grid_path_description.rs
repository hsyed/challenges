/// https://cses.fi/problemset/task/1625/
use crate::{Scanner, Writer};

pub fn solve(scan: &mut Scanner, out: &mut Writer) {
    let path_count = do_solve(&scan.next_line());
    out.println(path_count);
}

fn do_solve(template: &str) -> u32 {
    let mut grid = [[false; GRID_WIDTH]; GRID_HEIGHT];
    grid[0][0] = true;
    let template_bytes = template.as_bytes();

    dfs(0, 0, 0, &mut grid, template_bytes)
}

const GRID_HEIGHT: usize = 7;
const GRID_WIDTH: usize = 7;

/// Checks if moving to position (nx, ny) would create a barrier pattern.
/// A barrier occurs when the target cell has free cells on opposite sides,
/// which would split the grid into unreachable regions.
#[inline(always)]
fn would_create_barrier(grid: &[[bool; GRID_WIDTH]; GRID_HEIGHT], nx: usize, ny: usize) -> bool {
    let left_blocked = nx == 0 || grid[ny][nx - 1];
    let right_blocked = nx >= GRID_WIDTH - 1 || grid[ny][nx + 1];
    let up_blocked = ny == 0 || grid[ny - 1][nx];
    let down_blocked = ny >= GRID_HEIGHT - 1 || grid[ny + 1][nx];

    (!left_blocked && !right_blocked && up_blocked && down_blocked)
        || (left_blocked && right_blocked && !up_blocked && !down_blocked)
}

fn dfs(
    x: usize,
    y: usize,
    path_len: usize,
    grid: &mut [[bool; GRID_WIDTH]; GRID_HEIGHT],
    template: &[u8],
) -> u32 {
    // Destination is lower-left corner (0, 6)
    if x == 0 && y == GRID_HEIGHT - 1 {
        return if path_len == template.len() { 1 } else { 0 };
    }

    // template indexing misses a first move.
    let candidates: &[(isize, isize)] = match template[path_len] {
        b'?' => &[(0, -1), (1, 0), (0, 1), (-1, 0)],
        b'U' => &[(0, -1)],
        b'R' => &[(1, 0)],
        b'D' => &[(0, 1)],
        b'L' => &[(-1, 0)],
        _ => panic!("Invalid character in template"),
    };

    let mut count = 0;

    for &(dx, dy) in candidates {
        // Calculate new position (wrapping on underflow)
        let nx = x.wrapping_add_signed(dx);
        let ny = y.wrapping_add_signed(dy);

        // Check bounds and if cell is free
        if !(nx < GRID_WIDTH && ny < GRID_HEIGHT && !grid[ny][nx]) {
            continue;
        }

        // Critical pruning: Check barrier pattern BEFORE moving
        if would_create_barrier(grid, nx, ny) {
            continue;
        }

        grid[ny][nx] = true;
        count += dfs(nx, ny, path_len + 1, grid, template);
        grid[ny][nx] = false;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example() {
        let template = "??????R??????U??????????????????????????LD????D?";
        let result = do_solve(template);
        assert_eq!(result, 201, "Expected 201 paths, got {}", result);
    }
}
