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
use simple_matrix::Matrix;
use lazy_static::lazy_static;

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

type BlockTypeProto = Vec<bool>;
type BlockType = Matrix<bool>;
type StageType = Matrix<bool>;


lazy_static! {
/*
🍔🍔🧙🧙
🍔🍔🧙🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
 static ref SMASHBOY_BLOCK: BlockTypeProto =
	vec![true, true, false, false,
	 true, true, false, false,
	 false, false, false, false,
	 false, false, false, false];


/*
🧙🧙🍔🧙
🍔🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
static ref ORANGE_RICKY_BLOCK: BlockTypeProto = 
	vec![false, false, true, false,
	 true, true, true, false,
	 false, false, false, false,
	 false, false, false, false];

/*
🍔🧙🧙🧙
🍔🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
static ref BLUE_RICKY_BLOCK: BlockTypeProto = 
	vec![true, false, false, false,
	 true, true, true, false,
	 false, false, false, false,
	 false, false, false, false];

/*
🍔🍔🧙🧙
🧙🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
static ref CLEVELAND_Z_BLOCK: BlockTypeProto = 
	vec![true, true, false, false,
	 false, true, true, false,
	 false, false, false, false,
	 false, false, false, false];

/*
🧙🍔🍔🧙
🍔🍔🧙🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
static ref RHODE_ISLAND_Z_BLOCK: BlockTypeProto = 
	vec![false, true, true, false,
	 true, true, false, false,
	 false, false, false, false,
	 false, false, false, false];

/*
🍔🍔🍔🍔
🧙🧙🧙🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
static ref HERO_BLOCK: BlockTypeProto = 
	vec![true, true, true, true,
	false, false, false, false,
	false, false, false, false,
	false, false, false, false];

/*
🧙🍔🧙🧙
🍔🍔🍔🧙
🧙🧙🧙🧙
🧙🧙🧙🧙
*/
static ref TEEWEE_BLOCK: BlockTypeProto = 
	vec![false, true, false, false,
	 true, true, true, false,
	 false, false, false, false,
	 false, false, false, false];

static ref ZERO_BLOCK: BlockType =
	BlockType::from_iter(BLOCK_SIZE, BLOCK_SIZE, vec![false; BLOCK_SIZE * BLOCK_SIZE]);

static ref ZERO_STAGE: StageType=
	StageType::from_iter(STAGE_WIDTH, STAGE_HEIGHT, vec![false; STAGE_HEIGHT * STAGE_WIDTH]);

static ref BLOCKS: Vec<BlockTypeProto> =
	vec![SMASHBOY_BLOCK.to_vec(), ORANGE_RICKY_BLOCK.to_vec(),
		BLUE_RICKY_BLOCK.to_vec(), CLEVELAND_Z_BLOCK.to_vec(),
		RHODE_ISLAND_Z_BLOCK.to_vec(), HERO_BLOCK.to_vec(), TEEWEE_BLOCK.to_vec()];
}

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
	stage: StageType,
	current_block: BlockType,
	current_position: Pos
}

impl GameState {
	fn new() -> GameState {
		GameState {
			stage: ZERO_STAGE.clone(),
			current_block: ZERO_BLOCK.clone(),
			current_position: Pos{x: 0, y: 0}
		}
	}

  pub fn get_stage(&self, x: usize, y: usize) -> bool {
    *self.stage.get(x, y).unwrap()
  }

  pub fn get_current_block(&self, x: usize, y: usize) -> bool {
    *self.current_block.get(x, y).unwrap()
  }

	pub fn set_stage(&mut self, x: usize, y: usize, val: bool) {
		self.stage.set(x, y, val);
	}

}
 
fn can_move_down(game_state: &GameState) -> bool {
	for y in (0..BLOCK_SIZE).rev() {
		for x in 0..BLOCK_SIZE {
			// println!("block {:?}", game_state.current_block);
			if game_state.get_current_block(x, y) {

				if game_state.current_position.y + y == STAGE_HEIGHT-1 {
					println!("hit bottom");
					return false;
				} else if game_state.get_stage(
					game_state.current_position.x + x,
					game_state.current_position.y + y + 1) {
					println!("hit block");
					return false;
				}
			}
		}
	}

	true
}

fn advance_block(game_state: &mut GameState) {
	game_state.current_position.y += 1;
}

fn apply_block_to_stage(game_state: &mut GameState) {
	for x in 0..BLOCK_SIZE {
		for y in 0..BLOCK_SIZE {
			if game_state.get_current_block(x, y) {
				game_state.set_stage(
					game_state.current_position.x + x,
					game_state.current_position.y + y,
					true);
			}
		}
	}
}

fn is_full_row(game_state: &GameState, row: usize) -> bool {
	for x in 0..STAGE_WIDTH {
		if !game_state.get_stage(x,row) {
			return false;
		}
	}

	true
}

fn remove_row(game_state: &mut GameState, row: usize) {
	for x in 0..STAGE_WIDTH {
		game_state.set_stage(x, row, false);
	}
}

fn copy_line(game_state: &mut GameState, src: usize, dst: usize) {
	for x in 0..STAGE_WIDTH {
		game_state.set_stage(x, dst, game_state.get_stage(x, src));
	}
}

fn collapse_above(mut game_state: &mut GameState, row: usize) {
	for x in 0..row-1 {
		copy_line(&mut game_state, row-1-x, row-x);
	}
}

fn remove_full_rows(mut game_state: &mut GameState) {
	for y in 0..STAGE_HEIGHT {
		while is_full_row(&game_state, STAGE_HEIGHT-1-y) {
			remove_row(&mut game_state, STAGE_HEIGHT-1-y);
			collapse_above(&mut game_state, STAGE_HEIGHT-1-y);
		}
	}
}

fn ganerate_new_block(game_state: &mut GameState) {
	let mut rng = rand::thread_rng();
	let part = rng.gen_range(0, BLOCKS.len());
	
	// println!("new block {}", part);
	game_state.current_block = BlockType::from_iter(BLOCK_SIZE, BLOCK_SIZE,
		BLOCKS[part].clone());

	game_state.current_position = DEFAULT_START_POS;
}

fn check_collision(game_state: &GameState) -> bool {
	for x in 0..BLOCK_SIZE {
		for y in (0..BLOCK_SIZE).rev() {
			if game_state.get_current_block(x, y)
				&& game_state.get_stage(
					game_state.current_position.x + x,
					game_state.current_position.y + y) {
						return true;
			}
		}
	}

	false
}

fn move_left(game_state: &mut GameState) {
	for x in 0..BLOCK_SIZE {
		for y in 0..BLOCK_SIZE {
			if game_state.get_current_block(x, y) {
				if game_state.current_position.x + x == 0 {
					println!("hit left wall");
					return;
				} else if game_state.get_stage(
						game_state.current_position.x + x - 1,
						game_state.current_position.y + y) {
					println!("cannot move left");
					return;
				}
			}
		}
	}

	game_state.current_position.x -= 1;
}

fn move_right(game_state: &mut GameState) {
	for x in (0..BLOCK_SIZE).rev() {
		for y in 0..BLOCK_SIZE {
			if game_state.get_current_block(x, y) {
				if game_state.current_position.x + x == STAGE_WIDTH-1 {
					println!("hit right wall {}", game_state.current_position.x + x);
					return;
				} else if game_state.get_stage(
					game_state.current_position.x + x + 1,
					game_state.current_position.y + y) {
					println!("cannot move right");
					return;
				}
			}
		}
	}

	game_state.current_position.x += 1;
}

// fn copy_block(src: &BlockTypeProto, dst: &mut BlockTypeProto) {
// 	*dst = src.clone();
// 	// for i in 0..BLOCK_SIZE {
// 	// 	for j in 0..BLOCK_SIZE {
// 	// 		dst[i][j] = src[i][j];
// 	// 	}
// 	// }
// }

fn rotate_block(block: &mut BlockType) {

	// println!("rotate start {:?}", &block);
	// let mut tmp = Block::new();
	let tmp: BlockType = block.clone();
	// copy_block(&block, &mut tmp);
	// tmp = block.Clone();

	// move horizontal line to vertical
	for i in  0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			block.set(i, j, *tmp.get(i, BLOCK_SIZE-1-j).unwrap());
		}
	}

	// println!("after move horiz {:?}", &block);

	// shift to top left
	// tmp = block.clone();
	// // println!("rotate tmp {:?}", &tmp);
	
  // // tmp = block.Clone();
	// let mut empty_rows = 0;
	// let mut empty_cols = 0;
	// for i in 0..BLOCK_SIZE {
	// 	let mut is_empty = true;
	// 	for j in 0..BLOCK_SIZE {
	// 		if *tmp.get(i, j).unwrap() {
	// 			is_empty = false;
	// 			break;
	// 		}
	// 	}

	// 	if is_empty {
	// 		empty_rows += 1;
	// 	}
	// }

	// for i in  0..BLOCK_SIZE {
	// 	let mut is_empty = true;
	// 	for j in 0..BLOCK_SIZE {
	// 		if *tmp.get(i, j).unwrap() {
	// 			is_empty = false;
	// 			break;
	// 		}
	// 	}

	// 	if is_empty {
	// 		empty_cols += 1;
	// 	}
	// }

	// let tmp2: BlockType = [[false; BLOCK_SIZE]; BLOCK_SIZE];
	// *block = [[false; BLOCK_SIZE]; BLOCK_SIZE];
	// *block = [[false; BLOCK_SIZE]; BLOCK_SIZE];
	// println!("rotate false {:?}", &block);
	// println!("rotate tmp full {:?}", &tmp);
	// // copy_block(&tmp2, &mut block);
	// // // block = tmp2.Clone();
	// *block = ZERO_BLOCK.clone();
	// // println!("rows cols {} {}", empty_rows, empty_cols);
	// for i in  0..BLOCK_SIZE - empty_rows {
	// 	for j in 0..BLOCK_SIZE - empty_cols {
	// 		block.set(i, j, *tmp.get(i + empty_rows, j + empty_cols).unwrap());
	// 	}
	// }

	// println!("rotate end {:?}", &block);
}

fn can_rotate(game_state: &GameState) -> bool {
	let mut tmp : BlockType = game_state.current_block.clone();
  // copy_block(&game_state.current_block, &mut tmp);
  // tmp = game_state.current_block.Clone();

	rotate_block(&mut tmp);

	for i in 0..BLOCK_SIZE {
		for j in 0..BLOCK_SIZE {
			if *tmp.get(i, j).unwrap() {
				let x = game_state.current_position.x + j;
				let y = game_state.current_position.y + i;
				if x > STAGE_WIDTH-1 || y > STAGE_HEIGHT-1 || game_state.get_stage(i, j) {
					return false;
				}
			}
		}
	}

	true
}


impl App {
	fn render(&mut self, args: &RenderArgs, game_state: &GameState) {
		use graphics::*;

		let cell_width = args.window_size[0] / (STAGE_WIDTH as f64);
		let cell_height = args.window_size[1] / (STAGE_HEIGHT as f64);

		self.gl.draw(args.viewport(), |c, gl| {
			clear(BG_COLOR, gl); // clear screen

			// draw grid
			for x in 0..STAGE_WIDTH {
				for y in 0..STAGE_HEIGHT {
					let part = rectangle::square(x as f64 * cell_width,
						 y as f64 * cell_height, cell_width);
					let border = Rectangle::new_border(GRID_COLOR, 1.0);
					border.draw(part, &draw_state::DrawState::default(), c.transform, gl);

					let offset = cell_width / 6.0;
					let small_part = rectangle::square(x as f64 * cell_width + offset,
						 y as f64 * cell_height + offset, cell_width - offset*2.0);
					rectangle(BG_FILL_COLOR, small_part, c.transform, gl);
					
				}
			}
			
			// draw stage
			for x in 0..STAGE_WIDTH {
				for y in 0..STAGE_HEIGHT {
					if game_state.get_stage(x, y) {
						// fill
						let posx = x as f64 * cell_width;
						let posy = y as f64 * cell_height;
						let offset = cell_width / 6.0;
						let part = rectangle::square(posx + offset, posy + offset,
							 cell_width - offset * 2.0);
						rectangle(FILL_COLOR, part, c.transform, gl);

						// border
						let border_part = rectangle::square(x as f64 * cell_width,
							 y as f64 * cell_height, cell_width);
						let border = Rectangle::new_border(FILL_COLOR, 1.0);
						border.draw(border_part, &draw_state::DrawState::default(),
						 c.transform, gl);
					}
				}
			}

			// draw current block
			for x in 0..BLOCK_SIZE {
				for y in 0..BLOCK_SIZE {
					if game_state.get_current_block(x, y) {
						// fill
						let posx = (x + game_state.current_position.x) as f64 * cell_width;
						let posy = (y + game_state.current_position.y) as f64 * cell_height;
						let offset = cell_width / 6.0;
						let part = rectangle::square(posx + offset, posy + offset,
							 cell_width - offset*2.0);
						rectangle(FILL_COLOR, part, c.transform, gl);

						// border
						let border_part = rectangle::square(posx, posy, cell_width);
						let border = Rectangle::new_border(BORDER_COLOR, 1.0);
						border.draw(border_part, &draw_state::DrawState::default(),
						 c.transform, gl);

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
				if check_collision(&game_state) {
					println!("GAME OVER!");
				}
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

	let mut game_state = GameState::new();

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
			match key {
				Key::Left => move_left(&mut game_state),
				Key::Right => move_right(&mut game_state),
				Key::Down => {
					if can_move_down(&game_state ) {
						advance_block(&mut game_state);
					}
				}
				Key::Space => {
					if can_rotate(&game_state) {
						rotate_block(&mut game_state.current_block);
					}
				}
				_ => {}
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
