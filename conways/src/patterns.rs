/// Predefined patterns for Conway's Game of Life

use crate::game::Grid;

/// Load a glider pattern at position (x, y)
/// The glider travels diagonally
pub fn load_glider(grid: &mut Grid, x: usize, y: usize) {
    //  *
    //   **
    // **
    let pattern = [
        (1, 0),
        (2, 1),
        (0, 2), (1, 2), (2, 2),
    ];

    for (dx, dy) in pattern.iter() {
        let nx = (x + dx) % grid.width;
        let ny = (y + dy) % grid.height;
        grid.set(nx, ny, true);
    }
}

/// Load a blinker oscillator at position (x, y)
/// Period: 2
pub fn load_blinker(grid: &mut Grid, x: usize, y: usize) {
    // ***
    for dx in 0..3 {
        let nx = (x + dx) % grid.width;
        let ny = y % grid.height;
        grid.set(nx, ny, true);
    }
}

/// Load a toad oscillator at position (x, y)
/// Period: 2
pub fn load_toad(grid: &mut Grid, x: usize, y: usize) {
    //  ***
    // ***
    let pattern = [
        (1, 0), (2, 0), (3, 0),
        (0, 1), (1, 1), (2, 1),
    ];

    for (dx, dy) in pattern.iter() {
        let nx = (x + dx) % grid.width;
        let ny = (y + dy) % grid.height;
        grid.set(nx, ny, true);
    }
}

/// Load a beacon oscillator at position (x, y)
/// Period: 2
pub fn load_beacon(grid: &mut Grid, x: usize, y: usize) {
    // **
    // *
    //    *
    //   **
    let pattern = [
        (0, 0), (1, 0),
        (0, 1),
        (3, 2),
        (2, 3), (3, 3),
    ];

    for (dx, dy) in pattern.iter() {
        let nx = (x + dx) % grid.width;
        let ny = (y + dy) % grid.height;
        grid.set(nx, ny, true);
    }
}

/// Load a pulsar oscillator at position (x, y)
/// Period: 3, one of the most common oscillators
pub fn load_pulsar(grid: &mut Grid, x: usize, y: usize) {
    let quadrant = [
        (2, 0), (3, 0), (4, 0),
        (0, 2), (5, 2),
        (0, 3), (5, 3),
        (0, 4), (5, 4),
        (2, 5), (3, 5), (4, 5),
    ];

    // Apply all 4 quadrants symmetrically
    for (dx, dy) in quadrant.iter() {
        let positions = [
            (x + dx, y + dy),
            (x + 12 - dx, y + dy),
            (x + dx, y + 12 - dy),
            (x + 12 - dx, y + 12 - dy),
        ];

        for (nx, ny) in positions.iter() {
            let wrapped_x = nx % grid.width;
            let wrapped_y = ny % grid.height;
            grid.set(wrapped_x, wrapped_y, true);
        }
    }
}

/// Load a lightweight spaceship (LWSS) at position (x, y)
/// The LWSS travels horizontally
pub fn load_lwss(grid: &mut Grid, x: usize, y: usize) {
    //  *  *
    //      *
    //  *   *
    //   ****
    let pattern = [
        (1, 0), (4, 0),
        (5, 1),
        (1, 2), (5, 2),
        (2, 3), (3, 3), (4, 3), (5, 3),
    ];

    for (dx, dy) in pattern.iter() {
        let nx = (x + dx) % grid.width;
        let ny = (y + dy) % grid.height;
        grid.set(nx, ny, true);
    }
}

/// Load a pentadecathlon oscillator at position (x, y)
/// Period: 15
pub fn load_pentadecathlon(grid: &mut Grid, x: usize, y: usize) {
    let pattern = [
        (2, 0), (3, 0),
        (1, 1), (4, 1),
        (1, 2), (4, 2),
        (2, 3), (3, 3),
        (2, 4), (3, 4),
        (2, 5), (3, 5),
        (1, 6), (4, 6),
        (1, 7), (4, 7),
        (2, 8), (3, 8),
    ];

    for (dx, dy) in pattern.iter() {
        let nx = (x + dx) % grid.width;
        let ny = (y + dy) % grid.height;
        grid.set(nx, ny, true);
    }
}

/// Load multiple patterns across the grid for demonstration
pub fn load_demo_scene(grid: &mut Grid) {
    // Place various patterns at different positions
    load_glider(grid, 10, 10);
    load_blinker(grid, 30, 10);
    load_toad(grid, 40, 10);
    load_beacon(grid, 55, 10);
    load_lwss(grid, 70, 10);
    load_pulsar(grid, 20, 30);
    load_pentadecathlon(grid, 60, 35);

    // Add a few more gliders traveling in different directions
    load_glider(grid, 80, 80);
    load_glider(grid, 15, 85);
}
