extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;
use rand::Rng;

const STAGE_WIDTH: usize = 10;
const STAGE_HEIGHT: usize = 20;
const UPDATE_INTERVAL: f64 = 0.5;
const BLOCK_SIZE: usize = 4;
//const BG_OFFSET: f64 = 5.0;

const SCREEN_WIDTH: u32 = 250;
const SCREEN_HEIGHT: u32 = 500;

const BG_COLOR: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
const BG_FILL_COLOR: [f32; 4] = [0.4, 0.4, 0.4, 0.1];
const GRID_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.1];
const FILL_COLOR: [f32; 4] = [0.9, 0.9, 0.9, 1.0];
const BORDER_COLOR: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

type BlockType = [[bool; BLOCK_SIZE]; BLOCK_SIZE];

/*
🍔🍔🧙🧙
🍔🍔🧙🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
const SMASHBOY_BLOCK: BlockType = 
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
const ORANGE_RICKY_BLOCK: BlockType = 
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
const BLUE_RICKY_BLOCK: BlockType = 
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
const CLEVELAND_Z_BLOCK: BlockType = 
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
const RHODE_ISLAND_Z_BLOCK: BlockType = 
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
const HERO_BLOCK: BlockType = 
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
const TEEWEE_BLOCK: BlockType = 
	[[false, true, false, false],
	 [true, true, true, false],
	 [false, false, false, false],
	 [false, false, false, false]];

const BLOCKS: [BlockType; 7] = [SMASHBOY_BLOCK, ORANGE_RICKY_BLOCK, BLUE_RICKY_BLOCK, CLEVELAND_Z_BLOCK, RHODE_ISLAND_Z_BLOCK, HERO_BLOCK, TEEWEE_BLOCK];

#[derive(Copy, Clone)]
struct Pos {
	x: usize,
	y: usize
}

const DEFAULT_START_POS: Pos = Pos{x: 3, y: 0};

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

fn can_move_down(game_state: &GameState) -> bool {
	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			if game_state.current_block[BLOCK_SIZE-1-i][BLOCK_SIZE-1-j] {
				if game_state.current_position.y + BLOCK_SIZE-1-i == STAGE_HEIGHT-1 {
					//println!("block pos: {}, actual pos: {}", game_state.current_position.y, game_state.current_position.y + BLOCK_SIZE-1-i);
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
}

fn is_full_row(game_state: &GameState, row: usize) -> bool {
	for i in 0..STAGE_WIDTH {
		if !game_state.stage[row][i] {
			return false;
		}
	}

	return true;
}

fn remove_row(game_state: &mut GameState, row: usize) {
	for i in 0..STAGE_WIDTH {
		game_state.stage[row][i] = false;
	}
}

fn copy_line(game_state: &mut GameState, src: usize, dst: usize) {
	for i in 0..STAGE_WIDTH {
		game_state.stage[dst][i] = game_state.stage[src][i];
	}
}

fn collapse_above(mut game_state: &mut GameState, row: usize) {
	for i in 0..row-1 {
		copy_line(&mut game_state, row-1-i, row-i);
	}
}

fn remove_full_rows(mut game_state: &mut GameState) {
	for i in 0..STAGE_HEIGHT {
		while is_full_row(&game_state, STAGE_HEIGHT-1-i) {
			remove_row(&mut game_state, STAGE_HEIGHT-1-i);
			collapse_above(&mut game_state, STAGE_HEIGHT-1-i);
		}
	}
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

	game_state.current_position = DEFAULT_START_POS;
}

fn move_left(game_state: &mut GameState) {
	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			if game_state.current_block[j][i] {
				if game_state.current_position.x + i == 0 {
					println!("hit left wall");
					return;
				} else if game_state.stage[game_state.current_position.y + j][game_state.current_position.x + i - 1] {
					println!("cannot move left");
					return;
				}
			}
		}
	}

	game_state.current_position.x -= 1;
}

fn move_right(game_state: &mut GameState) {
	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			if game_state.current_block[j][BLOCK_SIZE-1-i] {
				if game_state.current_position.x + BLOCK_SIZE-1-i == STAGE_WIDTH-1 {
					println!("hit right wall {}", game_state.current_position.x + BLOCK_SIZE-1 + i);
					return;
				} else if game_state.stage[game_state.current_position.y + j][game_state.current_position.x + BLOCK_SIZE-i] {
					println!("cannot move right");
					return;
				}
			}
		}
	}

	game_state.current_position.x += 1;
}

fn copy_block(src: &BlockType, dst: &mut BlockType) {
	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			dst[i][j] = src[i][j];
		}
	}
}

fn rotate_block(block: &mut BlockType) {
	let mut tmp: BlockType = [[false; BLOCK_SIZE]; BLOCK_SIZE];
	copy_block(&block, &mut tmp);

	for i in  0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			block[j][i] = tmp[i][BLOCK_SIZE-1-j];
		}
	}
}

fn can_rotate(game_state: &GameState) -> bool {
	let mut tmp: BlockType = [[false; BLOCK_SIZE]; BLOCK_SIZE];
	copy_block(&game_state.current_block, &mut tmp);

	rotate_block(&mut tmp);

	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			if tmp[i][j] {
				let x = game_state.current_position.x + j;
				let y = game_state.current_position.y + i;
				if x > STAGE_WIDTH-1 || y > STAGE_HEIGHT-1 || game_state.stage[i][j] {
					return false;
				}
			}
		}
	}

	return true;
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
					border.draw(part, &draw_state::DrawState::default(), c.transform, gl);

					let offset = cell_width / 4.0;
					let small_part = rectangle::square(j as f64 * cell_width + offset, i as f64 * cell_height + offset, cell_width - offset);
					rectangle(BG_FILL_COLOR, small_part, c.transform, gl);
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
						let offset = cell_width / 4.0;
						let part = rectangle::square(posx + offset, posy + offset, cell_width - offset);
						rectangle(FILL_COLOR, part, c.transform, gl);

						let border_part = rectangle::square(j as f64 * cell_width, i as f64 * cell_height, cell_width);
						let border = Rectangle::new_border(BORDER_COLOR, 1.0);
						border.draw(border_part, &draw_state::DrawState::default(), c.transform, gl);

					}
				}
			}
		});
	}

	fn update(&mut self, args: &UpdateArgs, mut game_state: &mut GameState) {
		self.duration += args.dt;
		
		if self.duration > self.last_update + UPDATE_INTERVAL {
			self.last_update = self.duration;
			
			if can_move_down(&game_state ) {
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
	let mut window: Window = WindowSettings::new("Tetris 🧙🍔", [SCREEN_WIDTH, SCREEN_HEIGHT])
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
		if let Some(Button::Keyboard(key)) = e.press_args() {
			if key == Key::Left {
				move_left(&mut game_state)
			} else if key == Key::Right {
				move_right(&mut game_state)
			} else if key == Key::Space {
				if can_rotate(&game_state) {
					rotate_block(&mut game_state.current_block);
				}
			} else if key == Key::Down {
				if can_move_down(&game_state ) {
					advance_block(&mut game_state);
				}
			}
		}
		
		if let Some(args) = e.render_args() {
			app.render(&args, &game_state);
		}

		if let Some(args) = e.update_args() {
			app.update(&args, &mut game_state);
		}
	}
}
