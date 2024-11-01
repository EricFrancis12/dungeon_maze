use crate::Cell;

use std::{fs, io::Error};

pub fn write_maze_to_html_file(
    maze: &Vec<Vec<Cell>>,
    output_path: impl Into<String>,
) -> Result<(), Error> {
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

    let html = format!(
        r#"
            <!DOCTYPE html>
            <html lang="en">

            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Maze</title>

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

                    #gen-maze-btn {{
                        margin: 20px;
                        padding: 10px;
                        font-size: large;
                    }}

                    #maze {{
                        display: inline-grid;
                        grid-template-rows: repeat({}, 1fr);
                        grid-template-columns: repeat({}, 1fr);
                        border: solid 2px black;
                        margin: 30px;
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
                <div>
                    <div id="maze">
                        {}
                    </div>
                </div>
            </body>

            </html>
        "#,
        maze.len(),    // calculate maze height
        maze[0].len(), // calculate maze width
        cells.join("\n"),
    );

    fs::write(output_path.into(), html.trim())
}
