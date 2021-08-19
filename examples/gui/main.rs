use clap::{App, Arg};
use egui::Key::{ArrowDown, ArrowLeft, ArrowRight, ArrowUp};
use glium::glutin;
#[allow(unused_imports)]
use sgf_parser::*;

mod board_shader;
mod next_shader;
mod stone_shader;

struct GuiState {
    game: go::Game,
    board_number: u32,
    variation: u32,
}

impl GuiState {
    pub fn game(&self) -> &go::Game {
        &self.game
    }

    pub fn get_board(&self) -> Result<go::Board, go::Error> {
        self.game.get_board(self.board_number)
    }

    pub fn get_board_number(&self) -> u32 {
        self.board_number
    }
    pub fn get_variation(&self) -> u32 {
        self.variation
    }

    pub fn next_variation(&mut self) {
        if let Ok(board) = self.game.get_board(self.board_number) {
            if self.variation < board.get_variation_count()-1 {
                self.variation += 1;
            }
            else {
                self.variation = 0;
            }
        }
    }

    pub fn prev_variation(&mut self) {
        if let Ok(board) = self.game.get_board(self.board_number) {
            if self.variation > 0 {
                self.variation -= 1;
            } else {
                self.variation = board.get_variation_count() - 1;
            }
        }
    }

    pub fn first_board(&mut self) {
        self.board_number = 0;
        self.variation = 0;
    }

    pub fn next_board(&mut self) {
        if let Ok(board) = self.game.get_board(self.board_number) {
            if let Some(bn) = board.get_next(self.variation) {
                self.board_number = bn;
                self.variation = 0;
            }
        }
    }

    pub fn prev_board(&mut self) {
        if let Ok(board) = self.game.get_board(self.board_number) {
            self.board_number = board.get_prev();
        }
    }

    pub fn last_board(&mut self) {
        while let Ok(board) = self.game.get_board(self.board_number) {
            if let Some(bn) = board.get_next(0) {
                self.board_number = bn;
            } else {
                return;
            }
        }
    }

    pub fn last_move_number(&mut self) -> u32 {
        let mut bn = self.board_number;
        while let Ok(board) = self.game.get_board(bn) {
            match board.get_next(0) {
                Some(n) => bn = n,
                _ => return bn,
            }
        }
        bn
    }
}

fn create_display(event_loop: &glutin::event_loop::EventLoop<()>) -> glium::Display {
    let window_builder = glutin::window::WindowBuilder::new()
        .with_resizable(false)
        .with_inner_size(glutin::dpi::LogicalSize {
            width: 1100.0,
            height: 800.0,
        })
        .with_title("go");

    let context_builder = glutin::ContextBuilder::new()
        .with_depth_buffer(0)
        .with_srgb(true)
        .with_stencil_buffer(0)
        .with_vsync(true);

    glium::Display::new(window_builder, context_builder, &event_loop).unwrap()
}

fn main() -> Result<(), go::Error> {
    let matches = App::new("sgf_viewer")
        .version("0.1.0")
        .author("Bruce McIntosh <bruce.e.mcintosh@gmail.com>")
        .about("SGF viewer")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("SGF file name"),
        )
        .get_matches();

    let file_name = matches
        .value_of("file")
        .map(|f| f.to_string())
        .ok_or(go::Error::Other("No file parameter".to_string()))?;

    let event_loop = glutin::event_loop::EventLoop::with_user_event();
    let display = create_display(&&event_loop);

    let mut egui = egui_glium::EguiGlium::new(&display);

    println!("Loading board image");

    let board_tex = {
        let img = image::load(
            std::io::Cursor::new(&include_bytes!("resources/board-1.png")[..]),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();
        let img_dim = img.dimensions();
        let img = glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), img_dim);

        glium::texture::SrgbTexture2d::new(&display, img).unwrap()
    };

    println!("Loading black stone");

    let black_stone_tex = {
        let img = image::load(
            std::io::Cursor::new(&include_bytes!("resources/b.png")[..]),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();
        let img_dim = img.dimensions();
        let img = glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), img_dim);

        glium::texture::SrgbTexture2d::new(&display, img).unwrap()
    };

    println!("Loading white stone");

    let white_stone_tex = {
        let img = image::load(
            std::io::Cursor::new(&include_bytes!("resources/w.png")[..]),
            image::ImageFormat::Png,
        )
        .unwrap()
        .to_rgba8();
        let img_dim = img.dimensions();
        let img = glium::texture::RawImage2d::from_raw_rgba_reversed(&img.into_raw(), img_dim);

        glium::texture::SrgbTexture2d::new(&display, img).unwrap()
    };

    println!("Creating shaders");
    let mut ts = board_shader::Shader::new(&display);
    let mut ss = stone_shader::Shader::new(&display);
    let mut ns = next_shader::Shader::new(&display);

    println!("Parsing game");
    let mut gui_state = GuiState {
        game: go::Game::from_sgf_file(file_name).unwrap(),
        board_number: 0,
        variation: 0,
    };

    println!("Running GUI");

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            egui.begin_frame(&display);

            if egui.ctx().input().key_pressed(ArrowRight) {
                gui_state.next_board();
            }
            if egui.ctx().input().key_pressed(ArrowLeft) {
                gui_state.prev_board();
            }
            if egui.ctx().input().key_pressed(ArrowUp) {
                gui_state.next_variation();
            }
            if egui.ctx().input().key_pressed(ArrowDown) {
                gui_state.prev_variation();
            }

            let mut quit = false;

            let display_dim = display.get_framebuffer_dimensions();

            egui::SidePanel::left("my_side_panel", 300.0).show(egui.ctx(), |ui| {
                // egui::Window::new("my_side_panel").show(egui.ctx(), |ui| {
                // ui.heading("Hello World!");
                if ui.button("Quit").clicked() {
                    quit = true;
                }
                ui.add(egui::widgets::Separator::default().spacing(20.0));
                ui.horizontal(|ui| {
                    ui.label("White: ");
                    ui.label(gui_state.game().get_player_white());
                    ui.label(" (");
                    ui.label(gui_state.game().get_rank_white());
                    ui.label(" )");
                });
                ui.add(egui::widgets::Separator::default().spacing(20.0));

                ui.horizontal(|ui| {
                    ui.label("Black: ");
                    ui.label(gui_state.game().get_player_black());
                    ui.label(" (");
                    ui.label(gui_state.game().get_rank_black());
                    ui.label(" )");
                });
                ui.add(egui::widgets::Separator::default().spacing(20.0));

                ui.horizontal(|ui| {
                    if ui.button("<<").clicked() {
                        gui_state.first_board();
                    }
                    if ui.button("<").clicked() {
                        gui_state.prev_board()
                    }
                    if ui.button(">").clicked() {
                        gui_state.next_board()
                    }
                    if ui.button(">>").clicked() {
                        gui_state.last_board();
                    }
                    ui.label(gui_state.get_board_number().to_string());
                    ui.label(" of ");
                    ui.label(gui_state.last_move_number().to_string());
                });

                //                ui.spacing_mut().slider_width = 280.0;
                //                ui.add(
                //                    egui::Slider::new(&mut board_number, 0..=game.get_final_move_number())
                ////                        .text("Move")
                //                        .show_value(false)
                //                        .clamp_to_range(true)
                //                );
            });

            let (needs_repaint, shapes) = egui.end_frame(&display);

            *control_flow = if quit {
                glutin::event_loop::ControlFlow::Exit
            } else if needs_repaint {
                display.gl_window().window().request_redraw();
                glutin::event_loop::ControlFlow::Poll
            } else {
                glutin::event_loop::ControlFlow::Wait
            };

            {
                use glium::Surface as _;
                let mut target = display.draw();

                let clear_color = egui::Rgba::from_rgb(0.0, 0.0, 0.0);
                target.clear_color(
                    clear_color[0],
                    clear_color[1],
                    clear_color[2],
                    clear_color[3],
                );

                // draw things behind egui here

                ts.render(&mut target, &board_tex, display_dim);

                let board = gui_state.get_board().unwrap();

                for r in 0..board.get_size() {
                    for c in 0..board.get_size() {
                        if let Ok(p) = board.get_point(r, c) {
                            match p {
                                go::PointState::Filled {
                                    move_number,
                                    stone_color,
                                } => match stone_color {
                                    Color::White => ss.render(
                                        &mut target,
                                        &white_stone_tex,
                                        display_dim,
                                        r,
                                        c,
                                        stone_color,
                                        gui_state.get_board_number() > 0
                                            && board.get_last_move().get_number() == move_number,
                                    ),
                                    Color::Black => ss.render(
                                        &mut target,
                                        &black_stone_tex,
                                        display_dim,
                                        r,
                                        c,
                                        stone_color,
                                        gui_state.get_board_number() > 0
                                            && board.get_last_move().get_number() == move_number,
                                    ),
                                },
                                _ => {}
                            }
                        }
                    }
                }

                let next_boards = board.get_next_boards();
                let mut i = 0;
                for board_number in next_boards {
                    if let Ok(b) = gui_state.game().get_board(board_number) {
                        let m = b.get_last_move();
                        ns.render(
                            &mut target,
                            display_dim,
                            m.row(),
                            m.col(),
                            m.get_color(),
                            i == gui_state.get_variation(),
                        );
                        i += 1;
                    }
                }

                egui.paint(&display, &mut target, shapes);

                // draw things on top of egui here

                target.finish().unwrap();
            }
        };

        match event {
            // Platform-dependent event handlers to workaround a winit bug
            // See: https://github.com/rust-windowing/winit/issues/987
            // See: https://github.com/rust-windowing/winit/issues/1619
            glutin::event::Event::RedrawEventsCleared if cfg!(windows) => redraw(),
            glutin::event::Event::RedrawRequested(_) if !cfg!(windows) => redraw(),

            glutin::event::Event::WindowEvent { event, .. } => {
                egui.on_event(event, control_flow);
                display.gl_window().window().request_redraw(); // TODO: ask egui if the events warrants a repaint instead
            }

            _ => (),
        }
    });
}
