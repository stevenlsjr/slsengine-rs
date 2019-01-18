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
extern crate log;
extern crate env_logger;

#[macro_use]
extern crate failure;

use cgmath::prelude::*;
use cgmath::*;
use slsengine::{
    game::*,
    renderer::{
        backend_gl::{textures::*, *},
        *,
    },
    *,
};
use std::sync::Arc;

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
    use std::{fs, io::Write, path};

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
            info!("read configuration {:?}", parsed_conf);
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
            info!("wrote configuration to {:?}", f);
            default_conf
        }
    };
    Ok(conf)
}

/// Returns a vector of gl textures for the images
/// associated with a gltf model
fn model_textures_gl(
    model: &renderer::model::Model,
) -> Vec<Option<Arc<GlTexture>>> {
    let imports = model.imports();
    imports
        .document
        .images()
        .map(|i| -> Result<Arc<GlTexture>, failure::Error> {
            let data = &imports.images[i.index()];
            let mut tex = GlTexture::new()?;
            tex.load_from_image(data)?;

            Ok(Arc::new(tex))
        })
        .map(|res| match res {
            Ok(t) => Some(t),
            Err(e) => {
                error!("failed to create texture: {:?}", e);
                None
            }
        })
        .collect()
}

fn main() {
    use crate::sdl_platform::{platform, Platform};

    use std::time::*;
    let config = get_or_create_config().unwrap();

    env_logger::init();

    let (plt, gl_platform_builder) =
        platform().with_config(config).build_gl().unwrap();
    let _ctx = gl_platform_builder.gl_ctx();

    let Platform {
        window, event_pump, ..
    } = plt;
    let mut loop_state = MainLoopState::new();
    let path = system::asset_path().join("assets/models/DamagedHelmet.glb");
    info!("{:?}, {:?}", system::asset_path(), path);

    let mut renderer = GlRenderer::new(&window).unwrap();

    let mut timer = game::Timer::new(Duration::from_millis(1000 / 50));
    let mut world = game::EntityWorld::new(&renderer);

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
