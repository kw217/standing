//! Standing - standing wave demo using GFX
//!
//! Copyright 2017 Keith Wansbrough <keith@lochan.org>

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;

use gfx::traits::FactoryExt;
use gfx::Device;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        pos: [f32; 2] = "a_Pos",
        colour: [f32; 4] = "a_Colour",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
//        out: gfx::RenderTarget<ColorFormat> = "Target0",
        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

const SQUARE: [ Vertex; 8 ] = [
    Vertex { pos: [ -0.5, -0.5 ], colour: [1.0, 0.0, 0.0, 1.0] },
    Vertex { pos: [  0.5, -0.5 ], colour: [1.0, 1.0, 0.0, 1.0] },
    Vertex { pos: [  0.5,  0.2 ], colour: [0.0, 1.0, 0.0, 1.0] },
    Vertex { pos: [  0.0,  0.7 ], colour: [1.0, 1.0, 1.0, 1.0] },
    Vertex { pos: [ -0.5,  0.2 ], colour: [0.0, 0.0, 1.0, 1.0] },
    Vertex { pos: [  -0.6,  -0.2 ], colour: [0.2, 0.0, 1.0, 0.3] },
    Vertex { pos: [  0.2,  -0.2 ], colour: [0.2, 0.0, 1.0, 0.3] },
    Vertex { pos: [  0.0,  0.1 ], colour: [1.0, 1.0, 1.0, 0.3] },
];

const INDICES: &[u16] = &[
     0, 1, 2,
     2, 4, 0,
     2, 3, 4,
     5, 6, 7,
];


const CLEAR_COLOUR: [f32; 4] = [ 0.1, 0.1, 0.3, 1.0];

pub fn main() {
    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Standing waves".to_string())
        .with_dimensions(1024, 768)
        .with_vsync();
    let (window, mut device, mut factory, main_colour, mut main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();
    let pso = factory.create_pipeline_simple(
        r#"
            #version 150 core
            in vec2 a_Pos;
            in vec4 a_Colour;
            out vec4 v_Colour;

            void main() {
                v_Colour = a_Colour;
                gl_Position = vec4(a_Pos, 0.0, 1.0);
            }
        "#.as_bytes(),
        r#"
            #version 150 core
            in vec4 v_Colour;
            out vec4 Target0;

            void main() {
                Target0 = v_Colour;
            }
        "#.as_bytes(),
        pipe::new()
    ).unwrap();
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&SQUARE, INDICES);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        out: main_colour,
    };

    let mut running = true;
    while running {
        // fetch events
        events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
            match event {
                glutin::WindowEvent::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _) |
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(_width, _height) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                },
                _ => {},
            }
        });

        // draw a frame
        encoder.clear(&data.out, CLEAR_COLOUR);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();
    }
}
