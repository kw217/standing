//! Standing - standing wave demo using GFX
//!
//! Copyright 2017 Keith Wansbrough <keith@lochan.org>

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate time;

use gfx::traits::FactoryExt;
use gfx::Device;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        x: f32 = "a_X",  // x coordinate of wave [-1,1]
        p: f32 = "a_P",  // offset - vector P factor
        q: f32 = "a_Q",  // offset - vector Q factor
    }

    constant Locals {
        colour: [f32; 4] = "a_Colour",
        pv: [f32; 3] = "a_PV",
        align: f32 = "align",
        qv: [f32; 3] = "a_QV",
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
//        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
    }
}

const NUM_COMPONENTS: usize = 100;
const COMPONENT_PQS: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

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

            uniform Locals {
                vec4 a_Colour;
                vec3 a_PV;
                float align;
                vec3 a_QV;
            };

            in float a_X;
            in float a_P;
            in float a_Q;

            out vec4 v_Colour;

            void main() {
                v_Colour = a_Colour;
                vec3 base = vec3(a_X, 0.0, 0.0);
                vec3 pv = a_P * a_PV;
                vec3 qv = a_Q * a_QV;
                vec3 pos = base + pv + qv;
                gl_Position = vec4(base, 1.0);
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

    // Build a tube (prism).
    let mut vertices = vec![];
    let mut indices = vec![];
    for i in 0..NUM_COMPONENTS {
        // X simply ranges from -1 to 1.
        let x = -1.0 + 2.0 * (i as f32 / NUM_COMPONENTS as f32);
        // All vertices in cross-section at X.
        for pq in COMPONENT_PQS.iter() {
            let (p, q) = *pq;
            vertices.push( Vertex { x, p, q, })
        }

        // First vertex of this cross-section.
        let i1 = i * COMPONENT_PQS.len();

        if i == 0 {
            // First end-cap.
            indices.append(&mut vec![i1, i1 + 1, i1 + 2]);
            indices.append(&mut vec![i1 + 2, i1 + 3, i1]);
        }
        if i > 0 {
            // First vertex of last cross-section.
            let i0 = (i-1) * COMPONENT_PQS.len();
            // Sides of prism.
            for j in 0..COMPONENT_PQS.len() {
                let j1 = (j + 1) % COMPONENT_PQS.len();
                indices.append(&mut vec![i0 + j, i1 + j, i1 + j1]);
                indices.append(&mut vec![i1 + j1, i0 + j1, i0 + j]);
            }
        }
        if i == NUM_COMPONENTS - 1 {
            // Last end-cap.
            indices.append(&mut vec![i1, i1 + 3, i1 + 2]);
            indices.append(&mut vec![i1 + 2, i1 + 1, i1]);
        }
    }
    let indices_u16: Vec<u16> = indices.into_iter().map(|i| i as u16).collect();
    let indices_u16_slice: &[u16] = &indices_u16;
    println!("Vertices: {:?}", &vertices);
    println!("Indices: {:?}", indices_u16_slice);

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertices, indices_u16_slice);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        locals: factory.create_constant_buffer(1),
        out: main_colour,
    };

    let mut running = true;
    let mut last_t = time::precise_time_s();
    const REPORT_INTERVAL: f64 = 1.0;
    let mut next_report_t = last_t;
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

        // get instant
        let t = time::precise_time_s();

        // draw a frame
        let locals = Locals {
            colour: [ 1.0, 1.0, 0.0, 1.0 ],
            pv: [ 0.0, 0.1, 0.0 ],
            align: 0.0,
            qv: [ 0.0, 0.0, 0.1 ]
        };
        encoder.update_constant_buffer(&data.locals, &locals);
        encoder.clear(&data.out, CLEAR_COLOUR);
        encoder.draw(&slice, &pso, &data);
        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();

        // report fps
        let frame_time = t - last_t;
        last_t = t;
        if t > next_report_t {
            next_report_t += REPORT_INTERVAL;
            println!("Instantaneous FPS: {}", 1.0 / frame_time);
        }
    }
}
