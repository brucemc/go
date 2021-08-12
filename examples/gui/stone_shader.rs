#[cfg(dev)]
use glium;
use glium::{implement_vertex, uniform, Surface};

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

            float x = s + (position.x * h / w) / 20.0;
            x = x + ((col - 9.0) * h / w / 10.0 );

            float y = position.y / 20.0;
            y = y + ((row - 9.0) / 10.0 );

            gl_Position = vec4(x, y, 0.0, 1.0);
          }
        "#;

        let fragment_shader_src = r#"
        #version 140

        in vec2 v_tex_coords;

        uniform sampler2D colour_texture;
        uniform int mark;

        out vec4 fragColor;

        void main() {
           fragColor = texture(colour_texture, v_tex_coords);
           if (mark > 0) {
               float radius = 0.3;
               float boarder_thick = 0.05;
               float d = distance (vec2(0.49,0.51), v_tex_coords);
               float t1 = 1.0 - smoothstep(radius-boarder_thick, radius, d);
               float t2 = 1.0 - smoothstep(radius, radius+boarder_thick, d);
               if (mark == 1) {
                 fragColor = vec4(mix(fragColor.rgb, mix(vec3(0.0), fragColor.rgb, t1), t2), fragColor.a);
               }
               else {
                 fragColor = vec4(mix(fragColor.rgb, mix(vec3(0.8), fragColor.rgb, t1), t2), fragColor.a);
               }
               
               //fragColor = vec4(vec3(0.75)-fragColor.rgb, fragColor.a);
               //if (d < 0.3 && d > 0.2) {
               //  if (fragColor.r < 0.3) {
               //      fragColor = vec4(vec3(0.75)-fragColor.rgb, fragColor.a);
               //  }
               //  else {
               //      fragColor = vec4(fragColor.rgb-vec3(0.9), fragColor.a);
               //  }
               //}
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
        texture: &glium::texture::SrgbTexture2d,
        d: (u32, u32),
        r: usize,
        c: usize,
        mark: i32,
    ) {
        let uniforms = uniform! {
        colour_texture : texture,
        w : d.0 as f32,
        h : d.1 as f32,
        row : (18-r) as f32,
        col : c as f32,
        mark : mark};

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
