#[cfg(test)]
use crate::maze_from_xyz_seed;

#[test]
fn test_maze_from_xyz_seed() {
    let height = 4;
    let width = 4;
    let last_h = height - 1;
    let last_w = width - 1;

    for seed in [
        1234,
        5678,
        9012,
        0,
        1,
        2,
        3,
        407,
        1609,
        123456789,
        987654321,
        192837465,
        918273645,
        u32::MAX,
    ] {
        for i in -4..4 {
            let x = i;
            let y = i;
            let z = i;

            let maze = maze_from_xyz_seed(seed, height, width, x, y, z);

            // Ensure the right wall of the current maze matches the left wall of the neighboring maze to the right.
            let maze_x_plus_1 = maze_from_xyz_seed(seed, height, width, x + 1, y, z);
            for h in 0..height {
                assert_eq!(maze[h][last_w].wall_right, maze_x_plus_1[h][0].wall_left);
            }

            // Ensure the left wall of the current maze matches the right wall of the neighboring maze to the left.
            let maze_x_minus_1 = maze_from_xyz_seed(seed, height, width, x - 1, y, z);
            for h in 0..height {
                assert_eq!(maze[h][0].wall_left, maze_x_minus_1[h][last_w].wall_right);
            }

            // Ensure the ceiling of the current maze matches the floor of the neighboring maze below.
            let maze_y_plus_1 = maze_from_xyz_seed(seed, height, width, x, y + 1, z);
            for h in 0..height {
                for w in 0..width {
                    assert_eq!(maze[h][w].ceiling, maze_y_plus_1[h][w].floor);
                }
            }

            // Ensure the floor of the current maze matches the ceiling of the neighboring maze above.
            let maze_y_minus_1 = maze_from_xyz_seed(seed, height, width, x, y - 1, z);
            for h in 0..height {
                for w in 0..width {
                    assert_eq!(maze[h][w].floor, maze_y_minus_1[h][w].ceiling);
                }
            }

            // Ensure the bottom wall of the current maze matches the top wall of the neighboring maze above.
            let maze_z_plus_1 = maze_from_xyz_seed(seed, height, width, x, y, z + 1);
            for w in 0..width {
                assert_eq!(maze[last_h][w].wall_bottom, maze_z_plus_1[0][w].wall_top);
            }

            // Ensure the top wall of the current maze matches the bottom wall of the neighboring maze below.
            let maze_z_minus_1 = maze_from_xyz_seed(seed, height, width, x, y, z - 1);
            for w in 0..width {
                assert_eq!(maze[0][w].wall_top, maze_z_minus_1[last_h][w].wall_bottom);
            }
        }
    }
}
