extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use rand::Rng;

const STAGE_WIDTH: usize = 11;
const STAGE_HEIGHT: usize = 20;
const UPDATE_INTERVAL: f64 = 0.1;
//const BLOCK_ARRAY_SIZE: usize = 16;
const BLOCK_SIZE: usize = 4;

const BG_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const GRID_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const FILL_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];

/*
🍔🍔🧙🧙
🍔🍔🧙🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const SMASHBOY_BLOCK: [[bool; BLOCK_SIZE]; BLOCK_SIZE] = 
	[[true, true, false, false],
	 [true, true, false, false],
	 [false, false, false, false],
	 [false, false, false, false]];

/*
🧙🧙🍔🧙
🍔🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const ORANGE_RICKY_BLOCK: [[bool; BLOCK_SIZE]; BLOCK_SIZE] = 
	[[false, false, true, false],
	 [true, true, true, false],
	 [false, false, false, false],
	 [false, false, false, false]];

/*
🍔🧙🧙🧙
🍔🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const BLUE_RICKY_BLOCK: [[bool; BLOCK_SIZE]; BLOCK_SIZE] = 
	[[true, false, false, false],
	 [true, true, true, false],
	 [false, false, false, false],
	 [false, false, false, false]];

/*
🍔🍔🧙🧙
🧙🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const CLEVELAND_Z_BLOCK: [[bool; BLOCK_SIZE]; BLOCK_SIZE] = 
	[[true, true, false, false],
	 [false, true, true, false],
	 [false, false, false, false],
	 [false, false, false, false]];

/*
🧙🍔🍔🧙
🍔🍔🧙🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const RHODE_ISLAND_Z_BLOCK: [[bool; BLOCK_SIZE]; BLOCK_SIZE] = 
	[[false, true, true, false],
	 [true, true, false, false],
	 [false, false, false, false],
	 [false, false, false, false]];

/*
🍔🍔🍔🍔
🧙🧙🧙🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const HERO_BLOCK: [[bool; BLOCK_SIZE]; BLOCK_SIZE] = 
	[[true, true, true, true],
	[false, false, false, false],
	[false, false, false, false],
	[false, false, false, false]];

/*
🧙🍔🧙🧙
🍔🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const TEEWEE_BLOCK: [[bool; BLOCK_SIZE]; BLOCK_SIZE] = 
	[[false, true, false, false],
	 [true, true, true, false],
	 [false, false, false, false],
	 [false, false, false, false]];


const BLOCKS: [[[bool; BLOCK_SIZE]; BLOCK_SIZE]; 7] = [SMASHBOY_BLOCK, ORANGE_RICKY_BLOCK, BLUE_RICKY_BLOCK, CLEVELAND_Z_BLOCK, RHODE_ISLAND_Z_BLOCK, HERO_BLOCK, TEEWEE_BLOCK];

struct Pos {
	x: usize,
	y: usize
}

pub struct App {
	gl: GlGraphics, // OpenGL drawing backend.
	duration: f64,
	last_update: f64
}

pub struct GameState {
	stage: [[bool; STAGE_WIDTH]; STAGE_HEIGHT],
	current_block: [[bool; BLOCK_SIZE]; BLOCK_SIZE],
	current_position: Pos
}

fn can_move_block(game_state: &GameState) -> bool {
	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			if game_state.current_block[BLOCK_SIZE-1-i][BLOCK_SIZE-1-j] {
				if game_state.current_position.y + BLOCK_SIZE-1-i == STAGE_HEIGHT-1 {
					println!("block pos: {}, actual pos: {}", game_state.current_position.y, game_state.current_position.y + BLOCK_SIZE-1-i);
					println!("hit bottom");
					return false;
				} else if game_state.stage[game_state.current_position.y + BLOCK_SIZE-i][game_state.current_position.x + BLOCK_SIZE-1-j] {
					println!("hit block");
					return false;
				}
			}
		}
	}

	return true;
}

fn advance_block(game_state: &mut GameState) {
	game_state.current_position.y += 1;
}

fn apply_block_to_stage(game_state: &mut GameState) {
	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			if game_state.current_block[i][j] {
				game_state.stage[game_state.current_position.y + i][game_state.current_position.x + j] = true;
			}
		}
	}

	game_state.current_position = Pos{x: 0, y: 0};
}

fn remove_full_rows(_game_state: &mut GameState) {

}

fn ganerate_new_block(game_state: &mut GameState) {
	let mut rng = rand::thread_rng();
	let part = rng.gen_range(0, BLOCKS.len());
	
	println!("new block {}", part);
	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			game_state.current_block[i][j] = BLOCKS[part][i][j];
		}
	}
}

impl App {
	fn render(&mut self, args: &RenderArgs, game_state: &GameState) {
        use graphics::*;

		let cell_width = args.window_size[0] / (STAGE_WIDTH as f64);
		let cell_height = args.window_size[1] / (STAGE_HEIGHT as f64);

		self.gl.draw(args.viewport(), |c, gl| {
			clear(BG_COLOR, gl); // clear screen

			// draw grid
			for i in 0..STAGE_HEIGHT {
				for j in 0..STAGE_WIDTH {
					let part = rectangle::square(j as f64 * cell_width, i as f64 * cell_height, cell_width);
					let border = Rectangle::new_border(GRID_COLOR, 1.0);
					border.draw(part, &draw_state::DrawState::default(), c.transform, gl)
				}
			}
			
			// draw stage
			for i in 0..STAGE_HEIGHT {
				for j in 0..STAGE_WIDTH {
					if game_state.stage[i][j] {
						let part = rectangle::square(j as f64 * cell_width, i as f64 * cell_height, cell_width);
						rectangle(FILL_COLOR, part, c.transform, gl);
					}
				}
			}

			// draw current block
			for i in 0..BLOCK_SIZE {
				for j in 0..BLOCK_SIZE {
					if game_state.current_block[i][j] {
						let posx = (j + game_state.current_position.x) as f64 * cell_width;
						let posy = (i + game_state.current_position.y) as f64 * cell_height;
						let part = rectangle::square(posx, posy, cell_width);
						rectangle(FILL_COLOR, part, c.transform, gl);
					}
				}
			}
		});
	}

	fn update(&mut self, args: &UpdateArgs, mut game_state: &mut GameState) {
		self.duration += args.dt;
		
		if self.duration > self.last_update + UPDATE_INTERVAL {
			self.last_update = self.duration;
			
			if can_move_block(&game_state ) {
				advance_block(&mut game_state);
			} else {
				apply_block_to_stage(&mut game_state);
				remove_full_rows(&mut game_state);
				ganerate_new_block(&mut game_state);
			}
		}
	}
}

fn main() {
	// Change this to OpenGL::V2_1 if not working.
	let opengl = OpenGL::V3_2;

	// Create an Glutin window.
	let mut window: Window = WindowSettings::new("spinning-square", [300, 500])
		.graphics_api(opengl)
		.exit_on_esc(true)
		.build()
		.unwrap();

	let mut game_state = GameState {
		stage: [[false; STAGE_WIDTH]; STAGE_HEIGHT],
		current_block: [[false;  BLOCK_SIZE]; BLOCK_SIZE],
		current_position: Pos{x: 0, y: 0}
	};

	// Create a new game and run it.
	let mut app = App {
		gl: GlGraphics::new(opengl),
		duration: 0.0,
		last_update: 0.0
	};

	ganerate_new_block(&mut game_state);

	let mut events = Events::new(EventSettings::new());
	while let Some(e) = events.next(&mut window) {
		if let Some(args) = e.render_args() {
			app.render(&args, &game_state);
		}

		if let Some(args) = e.update_args() {
			app.update(&args, &mut game_state);
		}
	}
}
