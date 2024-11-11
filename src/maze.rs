use crate::{
    utils::rng::seed_to_rng,
    world::{Cell, CellSpecial, CellWall},
};

use bevy::utils::default;
use rand::{rngs::StdRng, Rng};
use std::collections::HashSet;

pub type Maze = Vec<Vec<Cell>>;

fn _maze_from_seed(seed: u32, height: usize, width: usize) -> Maze {
    let mut rng = seed_to_rng(seed);
    maze_from_rng(&mut rng, height, width)
}

pub fn maze_from_rng(rng: &mut StdRng, height: usize, width: usize) -> Maze {
    let mut maze: Maze = vec![
        vec![
            Cell {
                wall_top: CellWall::Solid,
                wall_bottom: CellWall::Solid,
                wall_left: CellWall::Solid,
                wall_right: CellWall::Solid,
                floor: CellWall::Solid,
                ceiling: CellWall::Solid,
                special: CellSpecial::None,
                ..default()
            };
            width
        ];
        height
    ];

    let mut x: usize = 0;
    let mut y: usize = 0;
    let mut visited: HashSet<(usize, usize)> = HashSet::new();
    let mut visit_history: Vec<(usize, usize)> = Vec::new();

    let area = height * width;
    while visited.len() < area {
        visited.insert((x, y));

        let mut unvisited_adj_cells: Vec<(usize, usize)> = Vec::new();

        // Check if unvisited cell to the left
        if x >= 1 && !visited.contains(&(x - 1, y)) {
            unvisited_adj_cells.push((x - 1, y));
        }
        // Check if unvisited cell to the right
        if x + 1 < width && !visited.contains(&(x + 1, y)) {
            unvisited_adj_cells.push((x + 1, y));
        }
        // Check if unvisited cell above
        if y >= 1 && !visited.contains(&(x, y - 1)) {
            unvisited_adj_cells.push((x, y - 1));
        }
        // Check if unvisited cell below
        if y + 1 < height && !visited.contains(&(x, y + 1)) {
            unvisited_adj_cells.push((x, y + 1));
        }

        // if no unvisited adjacent cells, pop the most recent (x,y) off of visit_history
        // and that becomes the new current (x,y)
        if unvisited_adj_cells.is_empty() {
            let (prev_x, prev_y) = visit_history
                .pop()
                .expect("expected previous (x,y), but encountered empty visit history");
            x = prev_x;
            y = prev_y;
            continue;
        }

        let rand_index: usize = rng.gen_range(0..unvisited_adj_cells.len());
        let (next_x, next_y) = unvisited_adj_cells[rand_index];

        visit_history.push((x, y));

        // Open walls based on the movement direction
        if next_x == x && next_y == y + 1 {
            // Moving down
            maze[y][x].wall_bottom = CellWall::None;
            maze[next_y][next_x].wall_top = CellWall::None;
        } else if next_x == x && next_y + 1 == y {
            // Moving up
            maze[y][x].wall_top = CellWall::None;
            maze[next_y][next_x].wall_bottom = CellWall::None;
        } else if next_x + 1 == x && next_y == y {
            // Moving left
            maze[y][x].wall_left = CellWall::None;
            maze[next_y][next_x].wall_right = CellWall::None;
        } else if next_x == x + 1 && next_y == y {
            // Moving right
            maze[y][x].wall_right = CellWall::None;
            maze[next_y][next_x].wall_left = CellWall::None;
        }

        x = next_x;
        y = next_y;
    }

    maze
}
