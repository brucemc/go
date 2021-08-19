#[cfg(dev)]
use glium;
use glium::{implement_vertex, uniform, Surface};
use sgf_parser::*;

#[derive(Copy, Clone)]
struct Vertex {
    position: [f32; 2],
    tex_coords: [f32; 2],
}

implement_vertex!(Vertex, position, tex_coords);

pub struct Shader {
    vertex_buffer: glium::VertexBuffer<Vertex>,
    indicies: glium::IndexBuffer<u32>,
    program: glium::Program,
}

impl Shader {
    pub fn new(display: &glium::Display) -> Shader {
        let shape = vec![
            Vertex {
                position: [-1.0, -1.0],
                tex_coords: [0.0, 0.0],
            },
            Vertex {
                position: [1.0, -1.0],
                tex_coords: [1.0, 0.0],
            },
            Vertex {
                position: [1.0, 1.0],
                tex_coords: [1.0, 1.0],
            },
            Vertex {
                position: [-1.0, 1.0],
                tex_coords: [0.0, 1.0],
            },
        ];

        let ind = vec![0, 1, 2, 0, 2, 3];

        let vertex_shader_src = r#"
          #version 450
          in vec2 position;
          in vec2 tex_coords;

          uniform float w;
          uniform float h;

          uniform float row;
          uniform float col;

          out vec2 v_tex_coords;

          void main() {
            v_tex_coords = tex_coords;

            float s = (w-h) / w;

            float x = s + (position.x * h / w) / 30.0;
            x = x + ((col - 9.0) * h / w / 10.0 );

            float y = position.y / 30.0;
            y = y + ((row - 9.0) / 10.0 );

            gl_Position = vec4(x, y, 0.0, 1.0);
          }
        "#;

        let fragment_shader_src = r#"
        #version 140
        in vec2 v_tex_coords;
        uniform bool white_stone;
        uniform bool selected;
        out vec4 fragColor;

        void main() {
           if (white_stone) {
             fragColor = vec4(0.9, 0.9, 0.9, 0.0);
           }
           else {
             fragColor = vec4(0.0, 0.0, 0.0, 0.0);
           }
           float radius = 0.5;
           float d = distance (vec2(0.49,0.51), v_tex_coords);
           if (d > radius) {
             fragColor = vec4(fragColor.rgb, 0.0);
           }
           else {
             float a = 0.7 * (1.0 - smoothstep(0.85*radius, radius, d));
             if (!selected) {
               a = 0.5 * a;
             }
               fragColor = vec4(fragColor.rgb, a);
           }
        }
    "#;

        let program =
            glium::Program::from_source(display, vertex_shader_src, fragment_shader_src, None)
                .unwrap();

        Shader {
            vertex_buffer: glium::VertexBuffer::immutable(display, &shape).unwrap(),
            indicies: glium::IndexBuffer::new(
                display,
                glium::index::PrimitiveType::TrianglesList,
                &ind,
            )
            .unwrap(),
            program,
        }
    }

    pub fn render(
        &mut self,
        target: &mut glium::Frame,
        display_dim: (u32, u32),
        r: u32,
        c: u32,
        move_color: Color,
        selected: bool,
    ) {
        let uniforms = uniform! {
        w : display_dim.0 as f32,
        h : display_dim.1 as f32,
        row : (18-r) as f32,
        col : c as f32,
        white_stone: move_color == Color::White,
        selected : selected};

        let params = glium::DrawParameters {
            blend: glium::draw_parameters::Blend::alpha_blending(),
            ..Default::default()
        };

        target
            .draw(
                &self.vertex_buffer,
                &self.indicies,
                &self.program,
                &uniforms,
                &params,
            )
            .unwrap();
    }
}
