use clap::{App, Arg};

use go;
use anyhow::Result;
use go::GoError;

fn main() -> Result<(), GoError> {
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

    let file_name = matches.value_of("file").map(|f| f.to_string()).ok_or(GoError::Other("No file parameter".to_string()))?;
    let game = go::Game::from_sgf_file(file_name)?;
    println!("{}", game.render_to_latex(50)?);
    let board = game.get_board(337).unwrap();
    println!("{}", board.to_ascii());
    Ok(())
}

