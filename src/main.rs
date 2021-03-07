extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
extern crate find_folder;
extern crate preferences;

use glutin_window::GlutinWindow as Window;
use opengl_graphics::{GlGraphics, OpenGL, GlyphCache, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::*;
use piston::window::WindowSettings;
use rand::Rng;
use simple_matrix::Matrix;
use lazy_static::lazy_static;
use preferences::{PreferencesMap, Preferences};
use std::fs::File;
use rusty_audio::Audio;


const HIGH_SCORE_PREF: &str = "highscore";

const STAGE_WIDTH: usize = 10;
const STAGE_HEIGHT: usize = 20;
const UPDATE_INTERVAL: f64 = 0.5;
const UPDATE_STEP: f64 = 0.05;
const UPDATE_LIMIT: f64 = 0.2;
const BLOCK_SIZE: usize = 4;
const SCREEN_WIDTH: u32 = 400;
const SCREEN_HEIGHT: u32 = 500;
const RENDER_STAGE_WIDTH: f64 = 250.0;
const RENDER_STAGE_HEIGHT: f64 = 500.0;

const BG_COLOR: [f32; 4] = [0.85, 0.85, 0.85, 1.0];
const BG_FILL_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.1];
const GRID_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 0.1];
const FILL_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const BORDER_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
const TEXT_COLOR: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

const LEVEL_UP_SCORE: i64 = 1000;
const ROW_SCORE: i64 = 100;
const BONUS_SCORE: i64 = 50;

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
	last_update: f64,
	
}

enum State {
	Running,
	LevelDone,
	GameOver
}

pub struct GameState {
	stage: StageType,
	current_block: BlockType,
	next_block: BlockType,
	current_position: Pos,
	score: i64,
	high_score: i64,
	level: i64,
	lines: i64,
	status: State,
	update_interval: f64,
}

impl GameState {
	fn new() -> GameState {
		GameState {
			stage: ZERO_STAGE.clone(),
			current_block: ZERO_BLOCK.clone(),
			next_block: ZERO_BLOCK.clone(),
			current_position: Pos{x: 0, y: 0},
			score: 0,
			high_score: 0,
			level: 1,
			lines: 0,
			status: State::Running,
			update_interval: UPDATE_INTERVAL
		}
	}

	fn get_stage(&self, x: usize, y: usize) -> bool {
		*self.stage.get(x, y).unwrap()
	}

	fn get_current_block(&self, x: usize, y: usize) -> bool {
		*self.current_block.get(x, y).unwrap()
	}

	fn get_next_block(&self, x: usize, y: usize) -> bool {
		*self.next_block.get(x, y).unwrap()
	}

	fn set_stage(&mut self, x: usize, y: usize, val: bool) {
		self.stage.set(x, y, val);
	}

	fn inc_score(&mut self, val: i64) {
		self.score += val;
	}
}
 
fn can_move_down(game_state: &GameState) -> bool {
	for y in (0..BLOCK_SIZE).rev() {
		for x in 0..BLOCK_SIZE {
			if game_state.get_current_block(x, y) {

				// hit bottom
				if game_state.current_position.y + y == STAGE_HEIGHT-1 {
					return false;
				// hit block
				} else if game_state.get_stage(
					game_state.current_position.x + x,
					game_state.current_position.y + y + 1) {
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

fn remove_full_rows(mut game_state: &mut GameState) -> bool {
	let mut lines = 0;
	for y in 0..STAGE_HEIGHT {
		while is_full_row(&game_state, STAGE_HEIGHT-1-y) {
			remove_row(&mut game_state, STAGE_HEIGHT-1-y);
			collapse_above(&mut game_state, STAGE_HEIGHT-1-y);
			game_state.inc_score(ROW_SCORE);
			lines += 1;
		}
	}

	game_state.lines += lines;

	if lines > 1 {
		game_state.inc_score(BONUS_SCORE);
	}

	return lines > 0;
}

fn generate_new_block(game_state: &mut GameState) {
	let mut rng = rand::thread_rng();
	let part = rng.gen_range(0, BLOCKS.len());
	
	game_state.current_block = game_state.next_block.clone();
	game_state.next_block = BlockType::from_iter(BLOCK_SIZE, BLOCK_SIZE,
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
					return; // hit left wall
				} else if game_state.get_stage(
						game_state.current_position.x + x - 1,
						game_state.current_position.y + y) {
					return; // hit left block
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
					return; // hit right wall
				} else if game_state.get_stage(
					game_state.current_position.x + x + 1,
					game_state.current_position.y + y) {
					return; // hit right block
				}
			}
		}
	}

	game_state.current_position.x += 1;
}

fn rotate_block(block: &mut BlockType) {
	let mut tmp: BlockType = block.clone();

	// move horizontal line to vertical
	for x in  0..BLOCK_SIZE {
		for y in 0..BLOCK_SIZE {
			block.set(y, x, *tmp.get(x, BLOCK_SIZE-1-y).unwrap());
		}
	}

	// shift to top left
	tmp = block.clone();
	let mut empty_rows = 0;
	let mut empty_cols = 0;
	for row in 0..BLOCK_SIZE {
		let mut is_empty = true;
		for col in 0..BLOCK_SIZE {
			if *tmp.get(row, col).unwrap() {
				is_empty = false;
				break;
			}
		}

		if is_empty {
			empty_rows += 1;
		} else {
			break;
		}
	}

	for col in 0..BLOCK_SIZE {
		let mut is_empty = true;
		for row in 0..BLOCK_SIZE {
			if *tmp.get(row, col).unwrap() {
				is_empty = false;
				break;
			}
		}

		if is_empty {
			empty_cols += 1;
		} else {
			break; 
		}
	}

	*block = ZERO_BLOCK.clone();
	for row in  0..BLOCK_SIZE - empty_rows {
		for col in 0..BLOCK_SIZE - empty_cols {
			block.set(row, col, *tmp.get(row + empty_rows, col + empty_cols).unwrap());
		}
	}
}

fn can_rotate(game_state: &GameState) -> bool {
	let mut tmp : BlockType = game_state.current_block.clone();

	rotate_block(&mut tmp);

	for x in 0..BLOCK_SIZE {
		for y in 0..BLOCK_SIZE {
			if *tmp.get(x, y).unwrap() {
				let x = game_state.current_position.x + x;
				let y = game_state.current_position.y + y;
				if x > STAGE_WIDTH-1 || y > STAGE_HEIGHT-1 {
					return false;
				} else if game_state.get_stage(x, y) {
					return false;
				}
			}
		}
	}

	true
}


impl App {
	fn render(&mut self, args: &RenderArgs, game_state: &GameState, glyph_cache: &mut GlyphCache) {
		use graphics::*;

		// let cell_width = args.window_size[0] / (STAGE_WIDTH as f64);
		// let cell_height = args.window_size[1] / (STAGE_HEIGHT as f64);
		let cell_width = RENDER_STAGE_WIDTH / (STAGE_WIDTH as f64);
		let cell_height = RENDER_STAGE_HEIGHT / (STAGE_HEIGHT as f64);

		self.gl.draw(args.viewport(), |context, gl| {
			clear(BG_COLOR, gl); // clear screen

			// draw grid
			for x in 0..STAGE_WIDTH {
				for y in 0..STAGE_HEIGHT {
					let part = rectangle::square(x as f64 * cell_width,
						 y as f64 * cell_height, cell_width);
					let border = Rectangle::new_border(GRID_COLOR, 1.0);
					border.draw(part, &draw_state::DrawState::default(), context.transform, gl);

					let offset = cell_width / 6.0;
					let small_part = rectangle::square(x as f64 * cell_width + offset,
						 y as f64 * cell_height + offset, cell_width - offset*2.0);
					rectangle(BG_FILL_COLOR, small_part, context.transform, gl);
					
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
						rectangle(FILL_COLOR, part, context.transform, gl);

						// border
						let border_part = rectangle::square(x as f64 * cell_width,
							 y as f64 * cell_height, cell_width);
						let border = Rectangle::new_border(FILL_COLOR, 1.0);
						border.draw(border_part, &draw_state::DrawState::default(),
						 context.transform, gl);
					}
				}
			}

			let grid_border_part = rectangle::rectangle_by_corners(
				0.0, 0.0, RENDER_STAGE_WIDTH, RENDER_STAGE_HEIGHT);
			let border = Rectangle::new_border(FILL_COLOR, 1.0);
			border.draw(grid_border_part, 
				&draw_state::DrawState::default(),
				context.transform,
				gl);

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
						rectangle(FILL_COLOR, part, context.transform, gl);

						// border
						let border_part = rectangle::square(posx, posy, cell_width);
						let border = Rectangle::new_border(BORDER_COLOR, 1.0);
						border.draw(border_part, &draw_state::DrawState::default(),
						 context.transform, gl);

					}
				}
			}

			// text
			text::Text::new_color(TEXT_COLOR, 16)
				.draw(format!("Score: {}", game_state.score).as_str(),
					glyph_cache,
					&context.draw_state,
					context.transform.trans(260.0, 50.0),
					gl).unwrap();

			
			text::Text::new_color(TEXT_COLOR, 16)
				.draw(format!("Level: {}", game_state.level).as_str(),
					glyph_cache,
					&context.draw_state,
					context.transform.trans(260.0, 70.0),
					gl).unwrap();

			text::Text::new_color(TEXT_COLOR, 16)
				.draw(format!("Lines: {}", game_state.lines).as_str(),
					glyph_cache,
					&context.draw_state,
					context.transform.trans(260.0, 90.0),
					gl).unwrap();

			text::Text::new_color(TEXT_COLOR, 16)
				.draw("Next:",
					glyph_cache,
					&context.draw_state,
					context.transform.trans(260.0, 130.0),
					gl).unwrap();
				
			text::Text::new_color(TEXT_COLOR, 16)
				.draw("High score:",
					glyph_cache,
					&context.draw_state,
					context.transform.trans(260.0, 400.0),
					gl).unwrap();
					
			text::Text::new_color(TEXT_COLOR, 16)
				.draw(format!("{}", game_state.high_score).as_str(),
					glyph_cache,
					&context.draw_state,
					context.transform.trans(270.0, 420.0),
					gl).unwrap();
					
			// draw next block
			for x in 0..BLOCK_SIZE {
				for y in 0..BLOCK_SIZE {
					if game_state.get_next_block(x, y) {
						// fill
						let posx = x as f64 * cell_width;
						let posy = y as f64 * cell_height;
						let offset = cell_width / 6.0;
						let part = rectangle::square(posx + offset, posy + offset,
							 cell_width - offset*2.0);
						rectangle(FILL_COLOR,
							part,
							context.transform.trans(270.0, 150.0),
							gl);

						// border
						let border_part = rectangle::square(posx, posy, cell_width);
						let border = Rectangle::new_border(BORDER_COLOR, 1.0);
						border.draw(border_part, 
							&draw_state::DrawState::default(),
							context.transform.trans(270.0, 150.0),
						 	gl);

					}
				}
			}

			let end_str = match game_state.status {
				State::LevelDone => "LEVEL UP",
				State::GameOver => "GAME OVER",
				_ => ""
			};

			text::Text::new_color(TEXT_COLOR, 16)
				.draw(format!("{}", end_str).as_str(),
					glyph_cache,
					&context.draw_state,
					context.transform.trans(270.0, 270.0),
					gl).unwrap();
		});
	}

	fn update(&mut self, args: &UpdateArgs, mut game_state: &mut GameState, audio: &mut Audio) {
		self.duration += args.dt;
		
		if self.duration > self.last_update + game_state.update_interval {
			self.last_update = self.duration;
			
			if can_move_down(&game_state ) {
				advance_block(&mut game_state);
			} else {
				apply_block_to_stage(&mut game_state);
				if remove_full_rows(&mut game_state) {
					audio.play("line");
				}

				if game_state.score >= LEVEL_UP_SCORE * game_state.level {
					game_state.status = State::LevelDone;
				}

				generate_new_block(&mut game_state);
				if check_collision(&game_state) {
					game_state.status = State::GameOver;
				}
			}
		}
	}
}

fn main() {
	// Change this to OpenGL::V2_1 if not working.
	let opengl = OpenGL::V3_2;

	// Create an Glutin window.
	let mut window: Window = WindowSettings::new(
		"Tetris 🧙🍔 v1.0", [SCREEN_WIDTH, SCREEN_HEIGHT])
		.graphics_api(opengl)
		.exit_on_esc(true)
		.build()
		.unwrap();

	let mut game = GameState::new();

	// Create a new game and run it.
	let mut app = App {
		gl: GlGraphics::new(opengl),
		duration: 0.0,
		last_update: 0.0
	};

	// font
	let assets = find_folder::Search::ParentsThenKids(3, 3)
		.for_folder("data").unwrap();
	let ref font = assets.join("font.ttf");

	if !font.exists() {
		panic!("Missing resource: data/font.ttf");
	}

	let mut glyph_cache = GlyphCache::new(
		font,
		(),
		TextureSettings::new()).unwrap();

	// preferences - high score
	let pref_path = "data/preferences.cfg";
	let mut prefs: PreferencesMap<String> = PreferencesMap::new();
	
	let file_result = File::open(pref_path);
	if file_result.is_ok() {
		let mut file = file_result.unwrap();
		let load_result =
			PreferencesMap::<String>::load_from(&mut file);
		if load_result.is_ok() {
			let hash = load_result.unwrap();
			if hash.contains_key(&HIGH_SCORE_PREF.to_string()) {
				game.high_score = hash[&HIGH_SCORE_PREF.to_string()].parse().unwrap();
			}
		}
	}

	// audio
	let mut audio = Audio::new();
	audio.add("move", "data/move.wav");
	audio.add("line", "data/line.wav");
	audio.add("levelup", "data/levelup.wav");
	audio.add("rotate", "data/rotate.wav");
	audio.add("gameover", "data/gameover.wav");

	generate_new_block(&mut game); // first next block is zero
	generate_new_block(&mut game);

	let mut events = Events::new(EventSettings::new());
	while let Some(e) = events.next(&mut window) {
		if let Some(Button::Keyboard(key)) = e.press_args() {
			match key {
				Key::Left => {
					move_left(&mut game);
					audio.play("move");
				},
				Key::Right => {
					move_right(&mut game);
					audio.play("move");
				},
				Key::Down => {
					if can_move_down(&game ) {
						advance_block(&mut game);
					}
				}
				Key::Space => {
					if can_rotate(&game) {
						rotate_block(&mut game.current_block);
						audio.play("rotate");
					}
				}
				_ => {}
			}
		}
		
		if let Some(args) = e.render_args() {
			app.render(&args, &game, &mut glyph_cache);
		}

		match game.status {
			State::Running => {
				if let Some(args) = e.update_args() {
					app.update(&args, &mut game, &mut audio);
				}
			},
			State::LevelDone => {
				if game.update_interval > UPDATE_LIMIT {
					game.update_interval -= UPDATE_STEP;
				}

				game.level += 1;
				game.status = State::Running;
				audio.play("levelup");
				
				if game.score > game.high_score {
					game.high_score = game.score;
					let mut file = File::create(pref_path).unwrap();
					prefs.insert(HIGH_SCORE_PREF.to_string(), game.high_score.to_string());
					prefs.save_to(&mut file).unwrap();
				}
			},
			State::GameOver => {
				if game.score > game.high_score {
					audio.play("gameover");
					game.high_score = game.score;
					let mut file = File::create(pref_path).unwrap();
					prefs.insert(HIGH_SCORE_PREF.to_string(), game.high_score.to_string());
					prefs.save_to(&mut file).unwrap();
				}
			}
		}
	}
}
