//! Standing - standing wave demo using GFX
//!
//! Copyright 2017 Keith Wansbrough <keith@lochan.org>

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate time;
extern crate cgmath;

use gfx::traits::FactoryExt;
use gfx::Device;
use cgmath::{Deg, Vector3, Matrix4, PerspectiveFov};

/// Number of segments in the string.
const NUM_COMPONENTS: usize = 5;

/// Cross-section polygon of the string, as (p,q).
const COMPONENT_PQS: [(f32, f32); 4] = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)];

/// P vector for cross-section.
const PV: [f32; 3] = [ 0.0, 0.2, 0.0 ];

/// Q vector for cross-section.
const QV: [f32; 3] = [ 0.0, 0.0, 2.5 ];

/// Background colour.
const CLEAR_COLOUR: [f32; 4] = [ 0.1, 0.1, 0.3, 1.0];

/// String colour.
const STRING_COLOUR: [f32; 4] = [ 1.0, 1.0, 0.0, 1.0 ];

/// Temporal frequency.
const TEMPORAL_FREQ_HZ: f64 = 0.2;

/// Spatial frequence.
const SPATIAL_FREQ_WAVES_PER_UNIT: f64 = 1.5;

/// Amplitude of waves (before model transformation).
const AMPLITUDE: f32 = 1.0;

/// Reporting interval (for console reporting of FPS etc).
const REPORT_INTERVAL_SEC: f64 = 1.0;

pub type ColorFormat = gfx::format::Rgba8;
pub type DepthFormat = gfx::format::DepthStencil;

gfx_defines!{
    vertex Vertex {
        x: f32 = "a_X",  // x coordinate of wave [-0.5,0.5]
        p: f32 = "a_P",  // offset - vector P factor
        q: f32 = "a_Q",  // offset - vector Q factor
        next_p: f32 = "a_NextP",  // offset - vector P factor for next vertex
        next_q: f32 = "a_NextQ",  // offset - vector Q factor for next vertex
    }

    constant Locals {
        // This struct is aligned to 4 x f32.
        model: [[f32; 4]; 4] = "u_Model", // transform into model coordinates
        view: [[f32; 4]; 4] = "u_View",  // projection from model into view coordinates
        colour: [f32; 4] = "a_Colour",  // colour of string
        pv: [f32; 3] = "a_PV",  // offset - vector P
        phase: f32 = "a_Phase",  // phase at x=0 (radians)
        qv: [f32; 3] = "a_QV",  // offset - vector Q
        freq: f32 = "a_Freq",  // spatial frequency (radians/unit)
        light: [f32; 3] = "u_Light", // light direction for Gouraud
        ampl: f32 = "a_Ampl",  // amplitude
    }

    pipeline pipe {
        vbuf: gfx::VertexBuffer<Vertex> = (),
        locals: gfx::ConstantBuffer<Locals> = "Locals",
        out: gfx::RenderTarget<ColorFormat> = "Target0",
//        out: gfx::BlendTarget<ColorFormat> = ("Target0", gfx::state::MASK_ALL, gfx::preset::blend::ALPHA),
//        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> =
//             gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

/// Compute projection matrix for window.
fn calc_projection(window: &glutin::Window) -> Matrix4<f32> {
    let (width, height) = window.get_inner_size_pixels().expect("No aspect!");
    println!("Width {} height {}", width, height);
    PerspectiveFov {
        fovy: Deg(15f32).into(),
        aspect: width as f32 / height as f32,
        near: 1.0,
        far: 20.0,
    }.into()
}

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
                mat4 u_Model;
                mat4 u_View;
                vec4 a_Colour;
                vec3 a_PV;
                float a_Phase;
                vec3 a_QV;
                float a_Freq;
                vec3 u_Light;
                float a_Ampl;
            };

            in float a_X;
            in float a_P;
            in float a_Q;
            in float a_NextP;
            in float a_NextQ;

            out vec4 v_Colour;
            flat out vec3 v_Normal;

            void main() {
                v_Colour = a_Colour;
                vec3 base = vec3(a_X, a_Ampl * sin((a_X * a_Freq) + a_Phase), 0.0);
                // tangent of the wave itself
                vec3 tangent = vec3(1.0, a_Ampl * a_Freq * cos((a_X * a_Freq) + a_Phase), 0.0);
                vec3 pv = a_P * a_PV;
                vec3 qv = a_Q * a_QV;
                vec3 dPos = pv + qv;
                vec3 nextPv = a_NextP * a_PV;
                vec3 nextQv = a_NextQ * a_QV;
                vec3 nextDPos = nextPv + nextQv;
                vec3 nextDelta = nextDPos - dPos;
                vec3 pos = base + dPos;
                v_Normal = normalize(cross(nextDelta, tangent));
                gl_Position = u_View * u_Model * vec4(pos, 1.0);
            }
        "#.as_bytes(),
        r#"
            #version 150 core
            in vec4 v_Colour;
            flat in vec3 v_Normal;
            out vec4 Target0;

            uniform Locals {
                mat4 u_Model;
                mat4 u_View;
                vec4 a_Colour;
                vec3 a_PV;
                float a_Phase;
                vec3 a_QV;
                float a_Freq;
                vec3 u_Light;
                float a_Ampl;
            };

            void main() {
                float brightness = dot(v_Normal, normalize(u_Light));
                vec4 dark_color = vec4(0.1, 0.1, 0.1, v_Colour.a);
                Target0 = mix(dark_color, v_Colour, brightness);
            }
        "#.as_bytes(),
        pipe::new()
    ).unwrap();

    // Build a tube (prism).
    let mut vertices = vec![];
    let mut indices = vec![];
    for i in 0..NUM_COMPONENTS {
        // X simply ranges from -0.5 to 0.5
        let x = -0.5 + (i as f32 / NUM_COMPONENTS as f32);
        // All vertices in cross-section at X.
        for j in 0..COMPONENT_PQS.len() {
            let (p, q) = COMPONENT_PQS[j];
            let (next_p, next_q) = COMPONENT_PQS[(j + 1) % COMPONENT_PQS.len()];
            vertices.push( Vertex { x, p, q, next_p, next_q, })
        }

        // First vertex of this cross-section.
        let i1 = i * COMPONENT_PQS.len();

        if i == 0 {
            // First end-cap.
            // ensure last vertex is i1
            indices.append(&mut vec![i1 + 1, i1 + 2, i1]);
            indices.append(&mut vec![i1 + 2, i1 + 3, i1]);
        }
        if i > 0 {
            // First vertex of last cross-section.
            let i0 = (i-1) * COMPONENT_PQS.len();
            // Sides of prism.
            for j in 0..COMPONENT_PQS.len() {
                let j1 = (j + 1) % COMPONENT_PQS.len();
                // ensure last vertex is i0 + j both times.
                indices.append(&mut vec![i1 + j, i1 + j1, i0 + j]);
                indices.append(&mut vec![i1 + j1, i0 + j1, i0 + j]);
            }
        }
        if i == NUM_COMPONENTS - 1 {
            // Last end-cap.
            // ensure last vertex is i1
            indices.append(&mut vec![i1 + 3, i1 + 2, i1]);
            indices.append(&mut vec![i1 + 2, i1 + 1, i1]);
        }
    }
    let indices_u16: Vec<u16> = indices.into_iter().map(|i| i as u16).collect();
    let indices_u16_slice: &[u16] = &indices_u16;

    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertices, indices_u16_slice);
    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        locals: factory.create_constant_buffer(1),
        out: main_colour,
    };

    let freq = (SPATIAL_FREQ_WAVES_PER_UNIT * 2.0 * std::f64::consts::PI) as f32;
    let ampl = AMPLITUDE;
    let local_to_world = Matrix4::from_nonuniform_scale(2.0, 0.2, 1.0);
    let world_to_camera = Matrix4::from_translation(Vector3::new(0.0, 0.15, -10.0));
    let mut projection = calc_projection(&window);
    let light: [f32; 3] = [-10.0, 10.0, 20.0];

    let mut running = true;
    let mut last_t = time::precise_time_s();
    let mut next_report_t = last_t;
    while running {
        // fetch events
        events_loop.poll_events(|glutin::Event::WindowEvent{window_id: _, event}| {
            match event {
                glutin::WindowEvent::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape), _) |
                glutin::WindowEvent::Closed => running = false,
                glutin::WindowEvent::Resized(_width, _height) => {
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut main_depth);
                    projection = calc_projection(&window);
                },
                _ => {},
            }
        });

        // get instant
        let t = time::precise_time_s();

        // draw a frame
        let phase = ((t * TEMPORAL_FREQ_HZ).fract() * 2.0 * std::f64::consts::PI) as f32;
        let locals = Locals {
            model: local_to_world.into(),
            view: (projection * world_to_camera).into(),
            colour: STRING_COLOUR,
            pv: PV,
            phase,
            qv: QV,
            freq,
            ampl,
            light,
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
            next_report_t += REPORT_INTERVAL_SEC;
            println!("Instantaneous FPS: {}", 1.0 / frame_time);
        }
    }
}
