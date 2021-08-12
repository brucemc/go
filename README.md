#go utils

Parse go games in SGF  (Smart Game Format) Files
into a form that can be used to render latex igo diagrams or a GUI application.

Build and run unit tests
- cargo build
- cargo test


Parse SGF file and write latex igo markup:
- cargo run --bin to_latex -- -f resources/The_59th_Judan_Title_Match_3rd_game.sgf

Parse SGF file and final board position in ASCII:
- cargo run --bin to_ascii -- -f ./resources/game.sgf


Parse SGF file and show in an example Open GL GUI:
- cargo run --example gui -- -f resources/The_59th_Judan_Title_Match_3rd_game.sgf
