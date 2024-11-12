use crate::{
    world::{chunk_from_xyz_seed, make_nei_chunks_xyz, CellWall, GRID_SIZE},
    SEED,
};

use chrono::Utc;
use std::fs;

#[test]
fn write_initial_chunks_to_html_file() {
    let output_path = "initial_chunks_map.html";

    let mut mazes_y_minus_1 = vec![];
    let mut mazes_y = vec![];
    let mut mazes_y_plus_1 = vec![];

    for (x, y, z) in make_nei_chunks_xyz((0, 0, 0), 1, 1, 1) {
        let maze = chunk_from_xyz_seed(SEED, x, y, z);
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

            for row in &maze.cells {
                for cell in row {
                    let mut classes = vec![];

                    if cell.wall_top != CellWall::None {
                        classes.push("top");
                    }
                    if cell.wall_bottom != CellWall::None {
                        classes.push("bottom");
                    }
                    if cell.wall_left != CellWall::None {
                        classes.push("left");
                    }
                    if cell.wall_right != CellWall::None {
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
        GRID_SIZE,
        GRID_SIZE,
        Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        button_elements.join("\n"),
        level_elements.join("\n"),
    );

    fs::write(output_path, html.trim()).unwrap();
}
