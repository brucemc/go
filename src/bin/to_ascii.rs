use clap::{App, Arg};

use go;
use anyhow::Result;
use go::Error;

fn main() -> Result<(), Error> {
    let matches = App::new("sgf_to_latex")
        .version("0.1.0")
        .author("Bruce McIntosh <bruce.e.mcintosh@gmail.com>")
        .about("Convert SGF file to latex")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("SGF file name"),
        )
        .get_matches();

    let file_name = matches.value_of("file").map(|f| f.to_string()).ok_or(Error::Other("No file parameter".to_string()))?;
    let game = go::Game::from_sgf_file(file_name)?;
    let board = game.get_board(game.get_final_move_number()).unwrap();
    println!("{}", board.to_ascii());
    Ok(())
}

