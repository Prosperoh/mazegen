use std::collections::HashSet;
use std::fmt;
use rand::seq::IteratorRandom;
use rand_pcg::Pcg64;
use rand::SeedableRng;

#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub struct Size {
    pub width: usize,
    pub height: usize,
}

type Coord = (usize, usize);

#[derive(Eq, PartialEq, Debug, Hash, Copy, Clone)]
pub enum TileDirection {
    NORTH, EAST, SOUTH, WEST
}

pub const ALL_TILE_DIRECTIONS: [&'static TileDirection; 4] = [
    &TileDirection::NORTH,
    &TileDirection::EAST,
    &TileDirection::SOUTH,
    &TileDirection::WEST
];

pub struct Cell {
    pub coord: Coord,
    walls: HashSet<TileDirection>,
}

impl Cell {
    pub fn new(coord: Coord) -> Self {
        Self { coord: coord, walls: HashSet::new() }
    }

    pub fn enable_wall(self: &mut Self, dir: &TileDirection) {
        self.walls.insert(*dir);
    }

    pub fn disable_wall(self: &mut Self, dir: &TileDirection) {
        self.walls.remove(dir);
    }

    pub fn is_wall_enabled(self: &Self, dir: &TileDirection) -> bool {
        self.walls.contains(dir)
    }
}

pub struct Maze {
    pub size: Size,
    cells: Vec<Vec<Cell>>,
}

impl Maze {
    pub fn new(size: &Size) -> Self {
        let mut cells = Vec::new();

        for i in 0..size.width {
            let mut row = Vec::new();
            for j in 0..size.height {
                let mut cell = Cell::new((i, j));
                // west edge
                if i == 0 {
                    cell.enable_wall(&TileDirection::WEST);
                }
                // east edge
                if i == size.height - 1 {
                    cell.enable_wall(&TileDirection::EAST);
                }
                // north edge
                if j == 0 {
                    cell.enable_wall(&TileDirection::NORTH);
                }
                // south edge
                if i == size.height - 1 {
                    cell.enable_wall(&TileDirection::SOUTH);
                }
                row.push(cell);
            }
            cells.push(row);
        }

        Self { size: *size, cells }
    }

    pub fn get_cell(self: &Self, coord: Coord) -> Option<&Cell> {
        self.cells.get(coord.0)?.get(coord.1)
    }

    pub fn get_mut_cell(self: &mut Self, coord: Coord) -> Option<&mut Cell> {
        self.cells.get_mut(coord.0)?.get_mut(coord.1)
    }

    fn get_neighbor_coords_and_dirs(self: &Self, coord: Coord) -> Vec<(Coord, TileDirection)> {
        let mut all_neighbors = vec![
            ((coord.0 + 1, coord.1), TileDirection::EAST),
            ((coord.0, coord.1 + 1), TileDirection::SOUTH),
        ];

        // checking coords to avoid usize overflow
        if coord.0 > 0 {
            all_neighbors.push(((coord.0 - 1, coord.1), TileDirection::WEST));
        }
        if coord.1 > 0 {
            all_neighbors.push(((coord.0, coord.1 - 1), TileDirection::NORTH));
        }

        all_neighbors
            .into_iter()
            .filter(|(coord, _)| self.is_valid_coord(coord))
            .collect()
    }

    pub fn get_neighbor_cells_and_dir(self: &Self, coord: Coord) -> Vec<(&Cell, TileDirection)> {
        self.get_neighbor_coords_and_dirs(coord)
            .into_iter()
            .map(|(coord, dir)| (self.get_cell(coord).unwrap(), dir))
            .collect()
    }

    fn get_opposite(direction: &TileDirection) -> TileDirection {
        match direction {
            TileDirection::NORTH => TileDirection::SOUTH,
            TileDirection::EAST => TileDirection::WEST,
            TileDirection::SOUTH => TileDirection::NORTH,
            TileDirection::WEST => TileDirection::EAST,
        }
    }

    fn get_mut_neighbor_cell_and_shared_wall(self: &mut Self, coord: Coord, direction: &TileDirection) -> Option<(&mut Cell, TileDirection)> {
        let (coord, dir) = self.get_neighbor_coords_and_dirs(coord)
            .into_iter()
            .find(|(_, dir)| dir == direction)?;

        match self.get_mut_cell(coord) {
            None => None,
            Some(cell) => Some((cell, Maze::get_opposite(&dir)))
        }
    }

    fn is_edge_wall(self: &Self, coord: Coord, direction: &TileDirection) -> bool {
        match direction {
            TileDirection::NORTH => coord.1 == 0,
            TileDirection::EAST => coord.0 == self.size.width - 1,
            TileDirection::SOUTH => coord.1 == self.size.height - 1,
            TileDirection::WEST => coord.0 == 0,
        }
    }

    pub fn is_wall_enabled(self: &Self, coord: Coord, direction: &TileDirection) -> bool {
        if self.is_edge_wall(coord, direction) {
            return true;
        }

        self.get_cell(coord).unwrap().is_wall_enabled(direction)
    }

    pub fn enable_wall(self: &mut Self, coord: Coord, direction: &TileDirection) {
        if self.is_edge_wall(coord, direction) {
            return;
        }

        self.get_mut_cell(coord).unwrap().enable_wall(direction);

        let (neighbor_cell, shared_wall_dir) = self.get_mut_neighbor_cell_and_shared_wall(coord, direction).unwrap();
        neighbor_cell.enable_wall(&shared_wall_dir);
    }

    pub fn disable_wall(self: &mut Self, coord: Coord, direction: &TileDirection) {
        if self.is_edge_wall(coord, direction) {
            return;
        }

        self.get_mut_cell(coord).unwrap().disable_wall(direction);

        let (neighbor_cell, shared_wall_dir) = self.get_mut_neighbor_cell_and_shared_wall(coord, direction).unwrap();
        neighbor_cell.disable_wall(&shared_wall_dir);
    }

    pub fn enable_all_walls(self: &mut Self) {
        for i in 0..self.size.width {
            for j in 0..self.size.height {
                for dir in ALL_TILE_DIRECTIONS.iter() {
                    let coord = (i, j);
                    if !self.is_edge_wall(coord, dir) {
                        self.enable_wall(coord, dir);
                    }
                }
            }
        }
    }

    pub fn disable_all_walls(self: &mut Self) {
        for i in 0..self.size.width {
            for j in 0..self.size.height {
                for dir in ALL_TILE_DIRECTIONS.iter() {
                    let coord = (i, j);
                    if !self.is_edge_wall(coord, dir) {
                        self.disable_wall(coord, dir);
                    }
                }
            }
        }
    }

    pub fn is_valid_coord(self: &Self, coord: &Coord) -> bool {
        // unsigned so no need to check if greater than zero
        coord.0 < self.size.width && coord.1 < self.size.height
    }
}

impl fmt::Display for Maze {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // first line contains upper walls
        let first_line = std::iter::repeat('_')
            .take(self.size.width * 2 - 1)
            .collect::<String>();

        writeln!(f, " {}", first_line)?;
        
        for j in 0..self.size.height {
            let mut first_line = String::new();
            let mut second_line = String::new();
            for i in 0..self.size.width - 1 {
                let cell = self.get_cell((i, j)).unwrap();

                // east wall line
                first_line.push(' ');
                if cell.is_wall_enabled(&TileDirection::EAST) {
                    first_line.push('#');
                } else {
                    first_line.push(' ');
                }

                // south wall line
                if cell.is_wall_enabled(&TileDirection::SOUTH) {
                    second_line.push('#');
                } else {
                    second_line.push(' ');
                }
                second_line.push('#');
            }
            writeln!(f, "|{} |", first_line)?;
            if j < self.size.height - 1 {
                writeln!(f, "|{}#|", second_line)?;
            }
        }

        Ok(())
    }
}

pub struct MazeGen {
    pub maze: Maze,
    left_to_visit: HashSet<Coord>,
    path_stack: Vec<Coord>,
}

impl MazeGen {

    pub fn new(size: &Size) -> Self {
        let mut left_to_visit = HashSet::new();
        for i in 0..size.width {
            for j in 0..size.height {
                left_to_visit.insert((i, j));
            }
        }
        
        Self { 
            maze: Maze::new(size),
            left_to_visit,
            path_stack: Vec::new(),
        }
    }

    fn get_valid_neighbor_coords_and_dirs(&self, coord: Coord) -> Vec<(Coord, TileDirection)> {
        self.maze.get_neighbor_coords_and_dirs(coord)
            .into_iter()
            .filter(|(coord, _)| self.left_to_visit.contains(coord))
            .collect()
    }

    // TODO: add seed
    pub fn generate(&mut self) {
        // reset maze
        self.maze.enable_all_walls();
        self.path_stack.clear();

        let mut rng = Pcg64::seed_from_u64(1512);

        let mut coord: Coord = (0, 0);
        self.left_to_visit.remove(&coord);

        while self.path_stack.len() > 0 || !self.left_to_visit.is_empty() {

            // one algo step: choose a direction or backtrack
            match self.get_valid_neighbor_coords_and_dirs(coord).into_iter().choose(&mut rng) {
                None => { coord = self.path_stack.pop().unwrap(); },
                Some((next_coord, dir)) => {
                    // remove wall between current and next cell
                    self.maze.disable_wall(coord, &dir);

                    self.path_stack.push(coord);
                    coord = next_coord;
                    self.left_to_visit.remove(&coord);
                },
            }
        }
    }
}


pub fn gen_maze(size: &Size) -> Maze {

    let mut mazegen = MazeGen::new(size);
    mazegen.generate();

    mazegen.maze
}
