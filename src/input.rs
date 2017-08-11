use *;
use std::io::{self, Write};

fn positions_join<'a, I: Iterator<Item = &'a (i8, i8)>>(input: I) -> String {
	let mut output = String::new();
	let mut first  = true;

	for pos in input {
		if first {
			first = false
		} else {
			output.push_str(", ");
		}

		output.push_str(&position_string(*pos));
	}

	output
}

pub fn main() {
	let mut board = make_board();

	loop {
		println!();
		#[cfg(not(feature = "white"))]
		match check_status(&mut board) {
			CheckStatus::None => {},
			CheckStatus::CheckMine(mate) => println!("BLACK CHECK{}\n", if mate { "MATE" } else { "ED" }),
			CheckStatus::CheckYour(mate) => println!("WHITE CHECK{}\n", if mate { "MATE" } else { "ED" }),
		}
		#[cfg(feature = "white")]
		match check_status(&mut board) {
			CheckStatus::None => {},
			CheckStatus::CheckMine(mate) => println!("WHITE CHECK{}\n", if mate { "MATE" } else { "ED" }),
			CheckStatus::CheckYour(mate) => println!("BLACK CHECK{}\n", if mate { "MATE" } else { "ED" }),
		}
		println!("{}", board::board_string(&board));

		println!("Commands: move(f), spawn, clear, reset, score, possible, all, best");
		print!("> ");
		io::stdout().flush().unwrap();

		let mut cmd = String::new();
		match io::stdin().read_line(&mut cmd) {
			Ok(0) => break,
			Ok(ok) => ok,
			Err(err) => {
				eprintln!("Error reading line: {}", err);
				break;
			}
		};

		let mut args = cmd.split_whitespace();
		let cmd = match args.next() {
			Some(cmd) => cmd,
			None => continue,
		};
		let args: Vec<_> = args.collect();

		macro_rules! usage {
			($n:expr, $usage:expr) => {
				if args.len() != $n {
					eprintln!($usage);
					eprintln!("Incorrect arguments");
					continue;
				}
			}
		}
		macro_rules! parse_pos {
			($input:expr) => {
				match parse_position($input) {
					Some(pos) => pos,
					None => {
						eprintln!("Invalid position");
						continue;
					}
				}
			}
		}
		match cmd {
			"move" | "movef" => {
				usage!(2, "move(f) <from> <to>");

				let force = cmd == "movef";

				let from = parse_pos!(args[0]);
				let to = parse_pos!(args[1]);

				if !force {
					let piece = *board_get(&board, from);

					if piece.is_mine() {
						eprintln!("Can't move with that piece! It's mine!");
						eprintln!("TIP: movef moves without checking first.");
						continue;
					}

					let mut found = false;
					for m in &piece.possible_moves(&board, from) {
						if *m == to {
							found = true;
						}
					}

					if !found {
						eprintln!("Can't move there!");
						eprintln!("TIP: movef moves without checking first.");
						continue;
					}
				}

				let (old_from, old_to, _) = board_move(&mut board, from, to);

				if !force {
					let possible = possible_moves(&board, true);
					if get_check(&board, false, &possible).is_some() {
						eprintln!("Can't move there! You'd place yourself in check!");
						eprintln!("TIP: movef moves without checking first.");

						board_set(&mut board, from, old_from);
						board_set(&mut board, to, old_to);
						continue;
					}
				}
			},
			"spawn" => {
				usage!(2, "spawn <position> <piece>");

				let pos = parse_pos!(args[0]);
				let piece = match args[1].parse() {
					Ok(piece) => piece,
					Err(_) => {
						eprintln!("No such piece");
						continue;
					}
				};

				board_set(&mut board, pos, piece);
			},
			"clear" => {
				usage!(0, "clear");

				for line in &mut board {
					for piece in line.iter_mut() {
						*piece = Piece::Empty;
					}
				}
			},
			"reset" => {
				usage!(0, "reset");

				board = board::make_board();
			},
			"score" => {
				usage!(0, "score");

				println!("{}", score(&board));
			}
			"possible" => {
				usage!(1, "possible <pos>");

				let pos = parse_pos!(args[0]);

				let possible = (*board_get(&board, pos)).possible_moves(&board, pos);
				if possible.is_empty() {
					println!("No possible moves");
					continue;
				}

				println!("{}", positions_join(possible.iter()));
			},
			"all" => {
				usage!(0, "all");

				let possible = possible_moves(&board, true);
				if possible.is_empty() {
					println!("No possible moves");
					continue;
				}

				for ((x, y), moves) in possible {
					if moves.is_empty() {
						continue;
					}

					let pos = position_string((x, y));
					println!("{}: {}", pos, positions_join(moves.iter()));
				}
			},
			"best" => {
				usage!(0, "best");

				#[cfg(feature = "cpuprofiler")]
				PROFILER.lock().unwrap().start("crappy-chess-minimax.profile").unwrap();

				let (score, from, to) = search(&mut board, true, 0, std::i32::MIN, std::i32::MAX);

				#[cfg(feature = "cpuprofiler")]
				PROFILER.lock().unwrap().stop().unwrap();

				board_move(&mut board, from, to);

				println!("Final Score: {}", score);

				println!("Move {} to {}", position_string(from), position_string(to));
			},
			_ => eprintln!("Unknown command"),
		}
	}
}