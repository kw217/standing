//! Standing - standing wave demo using GFX
//!
//! Copyright 2017 Keith Wansbrough <keith@lochan.org>
//!
//! Standard RH coordinates: XY is screen plane (X right, Y up), Z is towards viewer.

#[macro_use]
extern crate gfx;
extern crate gfx_window_glutin;
extern crate glutin;
extern crate time;
extern crate cgmath;
extern crate config;

mod app_config;

use gfx::traits::FactoryExt;
use gfx::{Device, Primitive, state};
use cgmath::{Deg, Matrix4, PerspectiveFov, SquareMatrix};

use app_config::AppConfig;


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
        out_depth: gfx::DepthTarget<gfx::format::DepthStencil> =
             gfx::preset::depth::LESS_EQUAL_WRITE,
    }
}

/// Compute projection matrix for window.
fn calc_projection(window: &glutin::Window, config: &AppConfig) -> Matrix4<f32> {
    let (width, height) = window.get_inner_size_pixels().expect("No aspect!");
    println!("Width {} height {}", width, height);
    PerspectiveFov {
        fovy: Deg(config.camera_fov_deg).into(),
        aspect: width as f32 / height as f32,
        near: config.camera_near,
        far: config.camera_far,
    }.into()
}

/// Build the geometry to render.
/// Returns vertices and indices.
fn build_geometry(config: &AppConfig) -> (Vec<Vertex>, Vec<u16>) {
    // Build a tube (prism).
    let mut vertices = vec![];
    let mut indices = vec![];
    for i in 0..config.num_components {
        // X simply ranges from -0.5 to 0.5
        let x = -0.5 + (i as f32 / config.num_components as f32);
        // All vertices in cross-section at X.
        for j in 0..config.component_pqs.len() {
            let pq = config.component_pqs[j];
            let next_pq = config.component_pqs[(j + 1) % config.component_pqs.len()];
            vertices.push( Vertex { x, p: pq[0], q: pq[1], next_p: next_pq[0], next_q: next_pq[1], })
        }

        // First vertex of this cross-section.
        let i1 = i * config.component_pqs.len();

        if i == 0 {
            // First end-cap.
            // ensure last vertex is i1
            // TODO get the normals correct for the endcaps somehow
            // TODO get the end cap shapes correct for config.component_pqs.len() != 3
            indices.append(&mut vec![i1 + 1, i1 + 2, i1]);
            indices.append(&mut vec![i1 + 2, i1 + 3, i1]);
        }
        if i > 0 {
            // First vertex of last cross-section.
            let i0 = (i-1) * config.component_pqs.len();
            // Sides of prism.
            for j in 0..config.component_pqs.len() {
                let j1 = (j + 1) % config.component_pqs.len();
                // ensure last vertex is i0 + j both times.
                indices.append(&mut vec![i1 + j, i1 + j1, i0 + j]);
                indices.append(&mut vec![i1 + j1, i0 + j1, i0 + j]);
            }
        }
        if i == config.num_components - 1 {
            // Last end-cap.
            // ensure last vertex is i1
            // TODO get the normals correct for the endcaps somehow
            // TODO get the end cap shapes correct for config.component_pqs.len() != 3
            indices.append(&mut vec![i1 + 3, i1 + 2, i1]);
            indices.append(&mut vec![i1 + 2, i1 + 1, i1]);
        }
    }
    let indices_u16: Vec<u16> = indices.into_iter().map(|i| i as u16).collect();
    (vertices, indices_u16)
}

pub fn main() {
    let config = app_config::AppConfig::new().expect("Unable to parse config file");

    let events_loop = glutin::EventsLoop::new();
    let builder = glutin::WindowBuilder::new()
        .with_title("Standing waves".to_string())
        .with_dimensions(1024, 768)
        .with_depth_buffer(24)
        .with_vsync();
    let (window, mut device, mut factory, main_colour, main_depth) =
        gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder, &events_loop);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let vertex_shader = include_bytes!("shader/standing_150.glslv");
    let fragment_shader = include_bytes!("shader/standing_150.glslf");
    let programs = factory.create_shader_set(vertex_shader, fragment_shader)
        .expect("Shader compilation failed");

    let pso = factory.create_pipeline_state(&programs,
        Primitive::TriangleList,
        // Cull the front face - guess my triangles or coordinates are backwards somehow!
//        state::Rasterizer { cull_face: state::CullFace::Front, ..state::Rasterizer::new_fill() },
        state::Rasterizer::new_fill().with_cull_back(),
        pipe::new()
    ).unwrap();

    let (vertices, indices) = build_geometry(&config);
    let (vertex_buffer, slice) = factory.create_vertex_buffer_with_slice(&vertices, &indices as &[u16]);

    let mut data = pipe::Data {
        vbuf: vertex_buffer,
        locals: factory.create_constant_buffer(1),
        out: main_colour,
        out_depth: main_depth,
    };

    let freq = (config.spatial_freq_waves_per_unit * 2.0 * std::f64::consts::PI) as f32;
    let ampl = config.amplitude;
    // invert so we specify camera position in world coords.
    let world_to_camera = Matrix4::from_translation(config.camera_pos.into()).invert().unwrap();
    let mut projection = calc_projection(&window, &config);
    let light: [f32; 3] = config.light_source_location;

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
                    gfx_window_glutin::update_views(&window, &mut data.out, &mut data.out_depth);
                    projection = calc_projection(&window, &config);
                },
                _ => {},
            }
        });

        // get instant
        let t = time::precise_time_s();
        let phase = ((t * config.temporal_freq_hz).fract() * 2.0 * std::f64::consts::PI) as f32;

        // draw a frame
        encoder.clear(&data.out, config.clear_colour);
        encoder.clear_depth(&data.out_depth, 1.0);

        for string in &config.strings {
            let local_to_world = Matrix4::from_translation(string.string_pos_1.into())
                * Matrix4::from_nonuniform_scale(string.string_scale[0], string.string_scale[1], string.string_scale[2]);
            let locals = Locals {
                model: local_to_world.into(),
                view: (projection * world_to_camera).into(),
                colour: string.string_colour,
                pv: config.pv,
                phase,
                qv: config.qv,
                freq,
                ampl,
                light,
            };
            encoder.update_constant_buffer(&data.locals, &locals);
            encoder.draw(&slice, &pso, &data);
        }

        encoder.flush(&mut device);
        window.swap_buffers().unwrap();
        device.cleanup();

        // report fps
        let frame_time = t - last_t;
        last_t = t;
        if t > next_report_t {
            next_report_t += config.report_interval_sec;
            println!("Instantaneous FPS: {}", 1.0 / frame_time);
        }
    }
}
