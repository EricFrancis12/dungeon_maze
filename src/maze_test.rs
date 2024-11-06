use chrono::Utc;
use core::panic;
use std::fs;

use crate::maze_from_xyz_seed;
use crate::{
    make_neighboring_chunks_xyz, maze::calc_maze_dims, CELL_SIZE, CHUNK_SIZE, DEFAULT_CHUNK_XYZ,
    SEED,
};

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

#[test]
fn write_initial_chunks_to_html_file() {
    let output_path = "maze.html";

    let (height, width) = calc_maze_dims(CHUNK_SIZE, CELL_SIZE);

    let mut mazes_y_minus_1 = vec![];
    let mut mazes_y = vec![];
    let mut mazes_y_plus_1 = vec![];

    for (x, y, z) in make_neighboring_chunks_xyz(DEFAULT_CHUNK_XYZ) {
        let maze = maze_from_xyz_seed(SEED, height, width, x, y, z);
        match y {
            -1 => mazes_y_minus_1.push(maze),
            0 => mazes_y.push(maze),
            1 => mazes_y_plus_1.push(maze),
            _ => panic!("expected y value of -1, 0, or 1 but got: {}", y),
        }
    }

    let maze_levels = vec![mazes_y_minus_1, mazes_y, mazes_y_plus_1];

    let mut level_elements = vec![];
    let mut button_elements = vec![];

    for i in 0..maze_levels.len() {
        let mazes = &maze_levels[i];
        let mut maze_elements = vec![];

        for maze in mazes {
            let mut cells = vec![];

            for row in maze {
                for cell in row {
                    let mut classes = vec![];

                    if cell.wall_top {
                        classes.push("top");
                    }
                    if cell.wall_bottom {
                        classes.push("bottom");
                    }
                    if cell.wall_left {
                        classes.push("left");
                    }
                    if cell.wall_right {
                        classes.push("right");
                    }

                    cells.push(format!(r#"<div class="cell {}"></div>"#, classes.join(" ")));
                }
            }

            maze_elements.push(format!(r#"<div class="maze">{}</div>"#, cells.join("\n")));
        }

        let level = i as i32 - 1;
        let active = if level == 0 { "active" } else { "" };

        level_elements.push(format!(
            r#"
                <div class="level-container {}" data-level="{}">
                    <div class="level">{}</div>
                </div>
            "#,
            active,
            level,
            maze_elements.join("\n"),
        ));

        button_elements.push(format!(
            r#"<button class="btn {}" data-level="{}">Y = {}</button>"#,
            active, level, level
        ));
    }

    let html = format!(
        r#"
            <!DOCTYPE html>
            <html lang="en">

            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Initial Chunk Map</title>

                <style>
                    * {{
                        margin: 0;
                        padding: 0;
                        box-sizing: border-box;
                    }}

                    body {{
                        text-align: center;
                    }}

                    h1 {{
                        margin: 20px;
                    }}

                    .btns-container {{
                        display: flex;
                        gap: 10px;
                        justify-content: center;
                        align-items: center;
                        margin: 20px;
                    }}

                    .btn {{
                        padding: 4px;
                    }}

                    .btn.active {{
                        background-color: green;
                    }}

                    .level-container {{
                        display: none;
                    }}

                    .level-container.active {{
                        display: block;
                    }}

                    .level {{
                        display: inline-grid;
                        grid-template-rows: repeat(3, 1fr);
                        grid-template-columns: repeat(3, 1fr);
                        margin: 30px;
                    }}

                    .maze {{
                        display: inline-grid;
                        grid-template-rows: repeat({}, 1fr);
                        grid-template-columns: repeat({}, 1fr);
                        margin: 10px;
                    }}

                    .cell {{
                        height: 50px;
                        width: 50px;
                    }}

                    .cell.top {{
                        border-top: solid 2px black;
                    }}

                    .cell.bottom {{
                        border-bottom: solid 2px black;
                    }}

                    .cell.left {{
                        border-left: solid 2px black;
                    }}

                    .cell.right {{
                        border-right: solid 2px black;
                    }}
                </style>
            </head>

            <body>
                <p>Generated at: {}</p>
                <div class="btns-container">
                    {}
                </div>
                <div id="chunks">
                    {}
                </div>

                <script>
                    for (const b of Array.from(document.querySelectorAll(".btn"))) {{
                        b.addEventListener("click", e => {{
                            for (const _b of Array.from(document.querySelectorAll(".btn"))) {{
                                _b.classList.remove("active");
                            }}
                            const {{ level }} = e.target.dataset;
                            for (const l of Array.from(document.querySelectorAll(".level-container"))) {{
                                if (l.dataset.level === level) {{
                                    l.classList.add("active");
                                    e.target.classList.add("active");
                                }} else {{
                                    l.classList.remove("active");
                                }}
                            }}
                        }});
                    }}
                </script>
            </body>

            </html>
        "#,
        height,
        width,
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        button_elements.join("\n"),
        level_elements.join("\n"),
    );

    fs::write(output_path, html.trim()).unwrap();
}
