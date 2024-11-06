use crate::{
    utils::rng::{rng_from_str, seed_to_rng},
    Cell, CellSpecial,
};

use rand::{rngs::StdRng, Rng};
use std::collections::HashSet;
use strum::IntoEnumIterator;

const WALL_BREAK_PROB: f64 = 0.2;

pub type Maze = Vec<Vec<Cell>>;

pub fn calc_maze_size(chunk_size: f32, cell_size: f32) -> usize {
    (chunk_size / cell_size) as usize
}

pub fn calc_maze_dims(chunk_size: f32, cell_size: f32) -> (usize, usize) {
    let maze_size = calc_maze_size(chunk_size, cell_size);
    (maze_size, maze_size) // (height, width)
}

fn _maze_from_seed(seed: u32, height: usize, width: usize) -> Maze {
    let mut rng = seed_to_rng(seed);
    maze_from_rng(&mut rng, height, width)
}

pub fn maze_from_xyz_seed(seed: u32, height: usize, width: usize, x: i64, y: i64, z: i64) -> Maze {
    let mut rng = rng_from_str(format!("{}-{}_{}_{}", seed, x, y, z));
    let mut maze = maze_from_rng(&mut rng, height, width);

    let last_h = height - 1;
    let last_w = width - 1;

    // left and right walls (x axis)
    for h in 0..height {
        let mut x_minus_1_rng = rng_from_str(fmt_seed_str(
            seed,
            (x - 1, y, z, last_w, h),
            (x, y, z, 0, h),
        ));
        if x_minus_1_rng.gen_bool(WALL_BREAK_PROB) {
            maze[h][0].wall_left = false;
        }

        let mut x_plus_1_rng = rng_from_str(fmt_seed_str(
            seed,
            (x, y, z, last_w, h),
            (x + 1, y, z, 0, h),
        ));
        if x_plus_1_rng.gen_bool(WALL_BREAK_PROB) {
            maze[h][last_w].wall_right = false;
        }
    }

    // top and bottom walls (z axis)
    for w in 0..width {
        let mut z_minus_1_rng = rng_from_str(fmt_seed_str(
            seed,
            (x, y, z - 1, w, last_h),
            (x, y, z, w, 0),
        ));
        if z_minus_1_rng.gen_bool(WALL_BREAK_PROB) {
            maze[0][w].wall_top = false;
        }

        let mut z_plus_1_rng = rng_from_str(fmt_seed_str(
            seed,
            (x, y, z, w, last_h),
            (x, y, z + 1, w, 0),
        ));
        if z_plus_1_rng.gen_bool(WALL_BREAK_PROB) {
            maze[last_h][w].wall_bottom = false;
        }
    }

    // ceiling and floor (y axis)
    for h in 0..height {
        for w in 0..width {
            let mut y_minus_1_rng =
                rng_from_str(fmt_seed_str(seed, (x, y - 1, z, w, h), (x, y, z, w, h)));
            if y_minus_1_rng.gen_bool(WALL_BREAK_PROB) {
                maze[h][w].floor = false;
            }

            let mut y_plus_1_rng =
                rng_from_str(fmt_seed_str(seed, (x, y, z, w, h), (x, y + 1, z, w, h)));
            if y_plus_1_rng.gen_bool(WALL_BREAK_PROB) {
                maze[h][w].ceiling = false;
            }
        }
    }

    let mut floored_cells: Vec<(usize, usize)> = Vec::new();
    for h in 0..height {
        for w in 0..width {
            if maze[h][w].floor {
                floored_cells.push((w, h));
            }
        }
    }

    let rand_floored_cell = |r: &mut StdRng, fc: &mut Vec<(usize, usize)>| {
        let i = r.gen_range(0..fc.len());
        let (w, h) = fc[i];
        fc.splice(i..i + 1, []);
        (w, h)
    };

    for spec in CellSpecial::iter() {
        if floored_cells.is_empty() {
            break;
        }

        if rng.gen_bool(spec.spawn_prob()) {
            let (w, h) = rand_floored_cell(&mut rng, &mut floored_cells);
            maze[h][w].special = spec;
        }
    }

    maze
}

pub fn maze_from_rng(rng: &mut StdRng, height: usize, width: usize) -> Maze {
    let mut maze: Maze = vec![
        vec![
            Cell {
                wall_top: true,
                wall_bottom: true,
                wall_left: true,
                wall_right: true,
                floor: true,
                ceiling: true,
                special: CellSpecial::None,
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
            maze[y][x].wall_bottom = false;
            maze[next_y][next_x].wall_top = false;
        } else if next_x == x && next_y + 1 == y {
            // Moving up
            maze[y][x].wall_top = false;
            maze[next_y][next_x].wall_bottom = false;
        } else if next_x + 1 == x && next_y == y {
            // Moving left
            maze[y][x].wall_left = false;
            maze[next_y][next_x].wall_right = false;
        } else if next_x == x + 1 && next_y == y {
            // Moving right
            maze[y][x].wall_right = false;
            maze[next_y][next_x].wall_left = false;
        }

        x = next_x;
        y = next_y;
    }

    maze
}

fn fmt_seed_str(
    seed: u32,
    greater_nei: (i64, i64, i64, usize, usize),
    less_nei: (i64, i64, i64, usize, usize),
) -> String {
    let (g_chunk_x, g_chunk_y, g_chunk_z, g_x, g_z) = greater_nei;
    let (l_chunk_x, l_chunk_y, l_chunk_z, l_x, l_z) = less_nei;
    format!(
        "{}-{}_{}_{}_{}_{}-{}_{}_{}_{}_{}",
        seed, g_chunk_x, g_chunk_y, g_chunk_z, g_x, g_z, l_chunk_x, l_chunk_y, l_chunk_z, l_x, l_z,
    )
}
