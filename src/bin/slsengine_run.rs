#![allow(dead_code)]

extern crate cgmath;
extern crate genmesh;
extern crate gl;
extern crate gltf;
extern crate sdl2;
extern crate slsengine;
extern crate stb_image;
extern crate toml;

#[macro_use]
extern crate failure;

use cgmath::prelude::*;
use cgmath::*;
use slsengine::renderer::{gl_renderer::*, material, objects, textures};
use slsengine::*;

// returns the [u, v] surface coordinates for a unit sphere.
fn uv_for_unit_sphere(pos: Vector3<f32>) -> [f32; 2] {
    use std::f32::consts::PI;
    let normal: Vector3<f32> = pos.normalize();
    let u = (normal.x.atan2(normal.z) / (2.0 * PI) + 0.5)
        .min(1.0)
        .max(0.0);
    let v = (normal.y * 0.5 + 0.5).min(1.0).max(0.0);

    [u, v]
}


fn get_or_create_config(
) -> Result<slsengine::config::PlatformConfig, failure::Error> {
    use slsengine::config::PlatformConfig;
    use std::{
        fs,
        io::{self, Write},
        path,
    };

    let pref_dir = sdl2::filesystem::pref_path("dangerbird", "slsengine")
        .map_err(&failure::Error::from)?;
    if let Err(_) = fs::metadata(&pref_dir) {
        fs::create_dir_all(&pref_dir).map_err(|_| {
            format_err!("could not create preferences directory {}", pref_dir)
        })?;
    }

    let full_path: path::PathBuf =
        [&pref_dir, "app_config.toml"].iter().collect();
    let conf: PlatformConfig = match fs::read(&full_path) {
        Ok(conf) => {
            let parsed_conf =
                toml::from_slice(&conf).map_err(&failure::Error::from)?;
            println!("read configuration {:?}", parsed_conf);
            parsed_conf
        }
        Err(_) => {
            let mut f =
                fs::File::create(full_path).map_err(&failure::Error::from)?;
            let default_conf = PlatformConfig::default();
            let v =
                toml::to_vec(&default_conf).map_err(&failure::Error::from)?;

            f.write(&v).map_err(|e| {
                format_err!("could not write default configuration!: {}", e)
            })?;
            println!("wrote configuration to {:?}", f);
            default_conf
        }
    };
    Ok(conf)
}

fn main() {
    use sdl_platform::{platform, OpenGLVersion, Platform};
    use std::time::*;
    let config = get_or_create_config().unwrap();

    let (plt, gl_platform_builder) =
        platform().with_config(config).build_gl().unwrap();
    let _ctx = gl_platform_builder.gl_ctx();

    let Platform {
        window, event_pump, ..
    } = plt;
    let mut loop_state = MainLoopState::new();
    let gltf_doc = gltf::Gltf::open("assets/models/DamagedHelmet.glb")
        .expect("could not load gltf model");
    let model = renderer::model::Model::from_gltf(&gltf_doc).unwrap();

    let mesh = &model.meshes[0].mesh;
    let mut renderer = GlRenderer::new(&window, mesh.clone()).unwrap();

   

    let mut timer = game::Timer::new(Duration::from_millis(1000 / 50));
    let mut world = game::EntityWorld::new();

    loop_state.is_running = true;
    let mut accumulator = Duration::from_secs(0);
    let fixed_dt = Duration::from_millis(100 / 6);
    while loop_state.is_running {
        {
            loop_state.handle_events(
                &window,
                &event_pump,
                &renderer,
                &mut world,
            );
        }
        let game::Tick { delta, .. } = timer.tick();
        accumulator += delta;

        let _ticks = Instant::now().duration_since(timer.start_instant());

        while accumulator >= fixed_dt {
            let ep = event_pump.borrow();

            world.update(fixed_dt, game::InputSources::from_event_pump(&ep));
            accumulator -= fixed_dt;
        }
        {
            renderer.on_update(delta, &world);
        }
        renderer.clear();

        renderer.render_scene(&world);

        window.gl_swap_window();
    }
}
