/// Core Conway's Game of Life grid logic with toroidal wrapping

#[derive(Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    cells: Vec<bool>,
}

impl Grid {
    /// Create a new empty grid with specified dimensions
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![false; width * height],
        }
    }

    /// Get cell index in flat array
    fn index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    /// Get the state of a cell at (x, y)
    pub fn get(&self, x: usize, y: usize) -> bool {
        self.cells[self.index(x, y)]
    }

    /// Set the state of a cell at (x, y)
    pub fn set(&mut self, x: usize, y: usize, alive: bool) {
        let idx = self.index(x, y);
        self.cells[idx] = alive;
    }

    /// Toggle the state of a cell at (x, y)
    pub fn toggle(&mut self, x: usize, y: usize) {
        let idx = self.index(x, y);
        self.cells[idx] = !self.cells[idx];
    }

    /// Clear all cells (set to dead)
    pub fn clear(&mut self) {
        self.cells.fill(false);
    }

    /// Wrap coordinate for toroidal topology
    /// Handles negative coordinates correctly
    fn wrap(&self, coord: isize, max: usize) -> usize {
        let max_i = max as isize;
        ((coord % max_i + max_i) % max_i) as usize
    }

    /// Count living neighbors for cell at (x, y) with toroidal wrapping
    pub fn count_alive_neighbors(&self, x: usize, y: usize) -> usize {
        let offsets = [
            (-1, -1),
            (-1, 0),
            (-1, 1),
            (0, -1),
            (0, 1),
            (1, -1),
            (1, 0),
            (1, 1),
        ];

        let x_i = x as isize;
        let y_i = y as isize;

        offsets
            .iter()
            .filter(|(dx, dy)| {
                let nx = self.wrap(x_i + dx, self.width);
                let ny = self.wrap(y_i + dy, self.height);
                self.get(nx, ny)
            })
            .count()
    }

    /// Compute the next generation using Conway's rules
    /// Returns a new Grid with the next state
    pub fn next_generation(&self) -> Grid {
        let mut next = Grid::new(self.width, self.height);

        for y in 0..self.height {
            for x in 0..self.width {
                let alive = self.get(x, y);
                let neighbors = self.count_alive_neighbors(x, y);

                // Conway's Game of Life rules:
                // 1. Any live cell with 2-3 neighbors survives
                // 2. Any dead cell with exactly 3 neighbors becomes alive
                // 3. All other cells die or stay dead
                let next_alive = match (alive, neighbors) {
                    (true, 2) | (true, 3) => true, // Survival
                    (false, 3) => true,            // Birth
                    _ => false,                    // Death
                };

                next.set(x, y, next_alive);
            }
        }

        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_positive() {
        let grid = Grid::new(10, 10);
        assert_eq!(grid.wrap(0, 10), 0);
        assert_eq!(grid.wrap(5, 10), 5);
        assert_eq!(grid.wrap(9, 10), 9);
    }

    #[test]
    fn test_wrap_negative() {
        let grid = Grid::new(10, 10);
        assert_eq!(grid.wrap(-1, 10), 9);
        assert_eq!(grid.wrap(-10, 10), 0);
        assert_eq!(grid.wrap(-11, 10), 9);
    }

    #[test]
    fn test_wrap_overflow() {
        let grid = Grid::new(10, 10);
        assert_eq!(grid.wrap(10, 10), 0);
        assert_eq!(grid.wrap(11, 10), 1);
        assert_eq!(grid.wrap(20, 10), 0);
    }

    #[test]
    fn test_blinker() {
        let mut grid = Grid::new(5, 5);
        // Vertical blinker
        grid.set(2, 1, true);
        grid.set(2, 2, true);
        grid.set(2, 3, true);

        let next = grid.next_generation();

        // Should be horizontal now
        assert!(next.get(1, 2));
        assert!(next.get(2, 2));
        assert!(next.get(3, 2));
        assert!(!next.get(2, 1));
        assert!(!next.get(2, 3));
    }

    #[test]
    fn test_block_still_life() {
        let mut grid = Grid::new(4, 4);
        // 2x2 block
        grid.set(1, 1, true);
        grid.set(2, 1, true);
        grid.set(1, 2, true);
        grid.set(2, 2, true);

        let next = grid.next_generation();

        // Should remain unchanged
        assert!(next.get(1, 1));
        assert!(next.get(2, 1));
        assert!(next.get(1, 2));
        assert!(next.get(2, 2));
    }
}
