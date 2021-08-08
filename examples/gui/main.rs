#[allow(unused_imports)]
use clap::{App, Arg};
use go::*;
use glium::glutin;

mod board_shader;
mod stone_shader;

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

fn main() -> Result<(), GoError> {

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

    let file_name = matches.value_of("file").map(|f| f.to_string()).ok_or(GoError::Other("No file parameter".to_string()))?;

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

        glium::texture::Texture2d::new(&display, img).unwrap()
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

        glium::texture::Texture2d::new(&display, img).unwrap()
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

        glium::texture::Texture2d::new(&display, img).unwrap()
    };

    println!("Creating shaders");
    let mut ts = board_shader::Shader::new(&display);
    let mut ss = stone_shader::Shader::new(&display);

    println!("Parsing game");
    let game =
        go::Game::from_sgf_file(file_name)
            .unwrap();
    let mut move_number: usize = 0;

    println!("Running GUI");

    event_loop.run(move |event, _, control_flow| {
        let mut redraw = || {
            egui.begin_frame(&display);

            let mut quit = false;

            let d = display.get_framebuffer_dimensions();

            egui::SidePanel::left("my_side_panel", 300.0).show(egui.ctx(), |ui| {
            // egui::Window::new("my_side_panel").show(egui.ctx(), |ui| {
                // ui.heading("Hello World!");
                if ui.button("Quit").clicked() {
                    quit = true;
                }
                ui.add(egui::widgets::Separator::default().spacing(20.0));
                ui.horizontal(|ui| {
                    ui.label("White: ");
                    ui.label(game.get_player_white());
                    ui.label(" (");
                    ui.label(game.get_rank_white());
                    ui.label(" )");
                });
                ui.add(egui::widgets::Separator::default().spacing(20.0));

                ui.horizontal(|ui| {
                    ui.label("Black: ");
                    ui.label(game.get_player_black());
                    ui.label(" (");
                    ui.label(game.get_rank_black());
                    ui.label(" )");
                });
                ui.add(egui::widgets::Separator::default().spacing(20.0));

                ui.horizontal(|ui| {
                    if ui.button("<<").clicked() {
                        move_number = 0;
                    }
                    if ui.button("<").clicked() {
                        if move_number > 0 {
                            move_number = move_number - 1;
                        }
                    }
                    if ui.button(">").clicked() {
                        if move_number < game.get_move_number() {
                            move_number = move_number + 1;
                        }
                    }
                    if ui.button(">>").clicked() {
                        move_number = game.get_move_number();
                    }
                    ui.label(move_number.to_string());
                    ui.label(" of ");
                    ui.label(game.get_move_number().to_string());
                    });

                ui.add(egui::Slider::new(&mut move_number, 0..=game.get_move_number()).text("Move"));

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

                ts.render(&mut target, &board_tex, d);
                // ss.render(&mut target, &black_stone_tex, d);
                let board = game.get_board(move_number).unwrap();

                for r in 0..19 {
                    for c in 0..19 {
                        if let Ok(p) = board.get_point(r, c) {
                            match p {
                                PointState::Filled {
                                    move_number:_,
                                    stone_color,
                                } => match stone_color {
                                    Color::White => {
                                        ss.render(&mut target, &white_stone_tex, d, r, c)
                                    }
                                    Color::Black => {
                                        ss.render(&mut target, &black_stone_tex, d, r, c)
                                    }
                                },
                                _ => {}
                            }
                        }
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
