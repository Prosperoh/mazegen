use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use mazegen::{Size, Maze, TileDirection, ALL_TILE_DIRECTIONS};

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
}

const WALL_THICKNESS: f64 = 1.0;
const CELL_SIZE: f64 = 25.0;
const CELL_MARGIN: f64 = 2.0;
const CELL_FULL_SIZE: f64 = (WALL_THICKNESS + CELL_MARGIN) * 2.0 + CELL_SIZE;

impl App {

    fn render(&mut self, args: &RenderArgs, maze: &Maze) {
        use graphics::*;

        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];

        let horizontal_wall = rectangle::rectangle_by_corners(0.0, 0.0, CELL_SIZE, WALL_THICKNESS);
        let vertical_wall = rectangle::rectangle_by_corners(0.0, 0.0, WALL_THICKNESS, CELL_SIZE);

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            for i in 0..maze.size.width {
                for j in 0..maze.size.height {

                    for direction in ALL_TILE_DIRECTIONS.iter() {
                        if maze.is_wall_enabled((i, j), direction) {
                            let (trans_x, trans_y, wall) = match direction {
                                TileDirection::NORTH => (CELL_MARGIN + WALL_THICKNESS, CELL_MARGIN, &horizontal_wall),
                                TileDirection::WEST => (CELL_MARGIN, CELL_MARGIN + WALL_THICKNESS, &vertical_wall),
                                TileDirection::SOUTH => (CELL_MARGIN + WALL_THICKNESS, CELL_MARGIN + WALL_THICKNESS * 2.0 + CELL_SIZE, &horizontal_wall),
                                TileDirection::EAST => (CELL_MARGIN + WALL_THICKNESS + CELL_SIZE, CELL_MARGIN + WALL_THICKNESS, &vertical_wall),
                            };

                            let (x, y) = (CELL_FULL_SIZE * (i as f64), CELL_FULL_SIZE * (j as f64));
                            let transform = c.transform
                                .trans(x, y)
                                .trans(trans_x, trans_y);

                            rectangle(RED, *wall, transform, gl);
                        }
                    }
                }
            }
        });
    }

    fn update(&mut self, _args: &UpdateArgs) {
        // empty
    }
}

fn main() {
    let maze = mazegen::gen_maze(&Size { width: 20, height: 20 });

    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let window_size = [
        maze.size.width as f64 * CELL_FULL_SIZE,
        maze.size.height as f64 * CELL_FULL_SIZE,
    ];

    let mut window: Window = WindowSettings::new("spinning-square", window_size)
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args, &maze);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}

