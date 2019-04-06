pub use super::{errors::*, program::*};

use super::{gl_materials::*, gl_mesh::*, objects::*, textures::*};
use crate::game;
use crate::renderer::*;
use cgmath::prelude::*;
use cgmath::*;
use core;
use failure;
use gl;
use log::*;
use sdl2::video::Window;
use std::{
    cell::{Cell, Ref, RefCell},
    path::Path,
    time::Instant,
};

use std::sync::Arc;

use std::borrow::Borrow;
use crate::game::component_stores::TryGetComponent;

pub type ManagedTexture = Arc<GlTexture>;
pub type ManagedTextureMaterial = material::Material<Arc<GlTexture>>;

pub struct Materials {
    default_material: Arc<ManagedTextureMaterial>,
    base_material_ubo: MaterialUbo,
}

/// the renderer backend for openGL
pub struct GlRenderer {
    camera: RefCell<Camera>,
    scene_program: PbrProgram,
    envmap_program: PbrProgram,
    sample_mesh: GlMesh,
    env_cube: GlMesh,
    pub materials: Materials,

    recompile_flag: Cell<Option<Instant>>,
}

fn create_scene_shaders() -> Result<(PbrProgram, PbrProgram), ShaderError> {
    use crate::platform_system::asset_path;
    let scene_program = program_from_sources(
        asset_path().join(Path::new("./assets/shaders/brdf.vert")),
        asset_path().join(Path::new("./assets/shaders/brdf.frag")),
        PbrShaderUniforms::default(),
    )?;

    let envmap_program = program_from_sources(
        asset_path().join(Path::new("./assets/shaders/envmap.vert")),
        asset_path().join(Path::new("./assets/shaders/envmap.frag")),
        PbrShaderUniforms::default(),
    )?;
    Ok((scene_program, envmap_program))
}

impl GlMesh {
    fn skybox_mesh() -> Result<Self, failure::Error> {
        use crate::renderer::Vertex;
        use genmesh::*;
        let cube = generators::Cube::new();
        let vertices: Vec<Vertex> = cube
            .vertex(|v| Vertex {
                position: v.pos.into(),
                ..Vertex::default()
            })
            .triangulate()
            .vertices()
            .collect();

        let indices: Vec<u32> = (0..vertices.len() as u32).collect();
        let mesh = Mesh { vertices, indices };
        let buffers = MeshBuffers::new()?;
        buffers.bind_mesh(&mesh)?;

        Ok(GlMesh { mesh, buffers })
    }
}

impl GlRenderer {
    pub fn new(window: &Window) -> Result<GlRenderer, RendererError> {
        let (width, height) = window.size();
        let perspective = PerspectiveFov {
            fovy: Deg(40.0).into(),
            aspect: width as f32 / height as f32,
            near: 0.1,
            far: 1000.0,
        };
        let (scene_program, envmap_program) =
            create_scene_shaders().map_err(RendererError::ShaderError)?;
        let mesh = {
            use crate::renderer::Vertex as V;
            use genmesh::generators::*;
            let icosphere = IcoSphere::new();
            let vertices: Vec<V> = icosphere
                .shared_vertex_iter()
                .map(|v| V {
                    position: v.pos.into(),
                    normal: v.normal.into(),
                    ..V::default()
                })
                .collect();

            let mesh = Mesh {
                vertices,
                indices: icosphere
                    .indexed_polygon_iter()
                    .flat_map(|tri| vec![tri.x, tri.y, tri.z])
                    .map(|i| i as u32)
                    .collect(),
            };

            GlMesh::with_mesh(mesh).map_err(|e| RendererError::Lifecycle {
                reason: format!("could not create placeholder mesh: {:?}", e),
            })?
        };

        let materials = GlRenderer::make_materials().map_err(|_| {
            RendererError::Lifecycle {
                reason: "could create material_obo".to_owned(),
            }
        })?;

        let env_cube =
            GlMesh::skybox_mesh().map_err(|_| RendererError::Lifecycle {
                reason: "could create skybox mesh".to_owned(),
            })?;

        let mut renderer = GlRenderer {
            scene_program,
            envmap_program,
            env_cube,
            sample_mesh: mesh,
            materials,
            camera: RefCell::new(Camera::new(perspective)),
            recompile_flag: Cell::new(None),
        };

        if let Err(e) = renderer.initialize() {
            return Err(RendererError::Lifecycle {
                reason: format!("could not initialize renderer: {:?}", e),
            });
        }

        renderer.on_resize((width, height));

        Ok(renderer)
    }

    pub fn projection(&self) -> Matrix4<f32> {
        self.camera.borrow().projection
    }

    fn initialize(&mut self) -> Result<(), failure::Error> {
        let ubo = &self.materials.base_material_ubo;
        for i in [&self.envmap_program, &self.scene_program].iter() {
            ubo.bind_to_program(i)
                .expect("could not set up program buffer");
        }

        Ok(())
    }

    fn make_materials() -> Result<Materials, ::failure::Error> {
        use crate::renderer::material::*;

        use failure::Error;
        let base_material: UntexturedMat = base::PLASTIC_RED;

        let default_material =
            Arc::new(base_material.transform_textures(|_, _| None));
        let base_material_ubo = MaterialUbo::new().map_err(&Error::from)?;

        Ok(Materials {
            default_material,
            base_material_ubo,
        })
    }

    pub fn rebuild_program(&mut self) {
        let (scene, skybox) = match create_scene_shaders() {
            Ok(programs) => programs,
            Err(e) => {
                error!("could not rebuild shaders: {}", e);
                return;
            }
        };

        if let Err(e) = self
            .materials
            .base_material_ubo
            .bind_to_program(&self.scene_program)
        {
            error!("failed to bind material {:?}", e);
        }

        self.scene_program = scene;
        self.envmap_program = skybox;
        {
            let camera = self.camera.borrow();

            self.scene_program.use_program();
            self.scene_program.bind_uniform(
                self.scene_program.uniforms().projection,
                &camera.projection,
            );
            self.envmap_program.use_program();

            self.envmap_program.bind_uniform(
                self.envmap_program.uniforms().projection,
                &camera.projection,
            );
        }

        let ubo = &self.materials.base_material_ubo;
        for i in &[&self.envmap_program, &self.scene_program] {
            ubo.bind_to_program(i)
                .unwrap_or_else(|e| println!("error {:?}", e));
        }

        info!("build new shader program {:#?}", self.scene_program);
    }

    #[inline]
    pub fn scene_program(&self) -> &PbrProgram {
        &self.scene_program
    }

    fn draw_entities<CS: TryGetComponent>(&self, scene: &game::EntityWorld<Self, CS>) {
        use crate::math::*;
        use std::ptr;
        let program = &self.scene_program;
        let uniforms = program.uniforms();

        program.use_program();
        unsafe { gl::Enable(gl::CULL_FACE) };

        let cam_view = scene.main_camera.transform();
        let light_positions: &[Vec3] = &[
            vec3(10.0, 10.0, 10.0),
            vec3(10.0, -10.0, 10.0),
            vec3(-10.0, -10.0, 10.0),
            vec3(-10.0, 10.0, 10.0),
        ];
        let xformed_light_positions: Vec<Vec3> = light_positions
            .iter()
            .map(|v| (cam_view * v.extend(1.0)).xyz())
            .collect();
        let light_pos_ptr = xformed_light_positions.as_ptr();

        uniforms
            .light_positions
            .map(|id| unsafe {
                gl::Uniform3fv(id as _, 4, light_pos_ptr as *const _);
            })
            .unwrap_or(());
    }
}

impl Renderer for GlRenderer {
    type Texture = ManagedTexture;
    type Mesh = GlMesh;
    fn clear(&self) {
        unsafe {
            gl::ClearColor(0.6, 0.0, 0.8, 1.0);
            gl::Enable(gl::DEPTH_TEST);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    fn camera(&self) -> Ref<Camera> {
        self.camera.borrow()
    }

    fn set_clear_color(&mut self, color: ColorRGBA) {
        unsafe {
            gl::ClearColor(color.r, color.g, color.b, color.a);
        }
    }

    fn on_resize(&self, size: (u32, u32)) {
        self.camera.borrow_mut().on_resize(size);
        let (width, height) = size;
        self.scene_program.use_program();
        let projection = &self.camera.borrow().projection;

        self.scene_program
            .bind_uniform(self.scene_program.uniforms().projection, projection);
        self.envmap_program.use_program();
        self.envmap_program.bind_uniform(
            self.envmap_program.uniforms().projection,
            projection,
        );

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    fn on_update<CS: TryGetComponent>(
        &mut self,
        _delta_time: ::std::time::Duration,
        _world: &game::EntityWorld<Self, CS>,
    ) {
        if let Some(_t) = self.recompile_flag.get() {
            self.rebuild_program();
            self.recompile_flag.set(None);
        }
    }

    fn flag_shader_recompile(&self) {
        if self.recompile_flag.get().is_none() {
            self.recompile_flag.set(Some(Instant::now()))
        }
    }

    fn render_scene<CS: TryGetComponent>(&self, scene: &game::EntityWorld<Self, CS>) {
        use std::ptr;

        use crate::math::*;
        let _program = self.scene_program();
        let cam_view = scene.main_camera.transform();

        let _uniforms = &self.scene_program.uniforms();

        {
            let GlMesh {
                ref buffers,
                ref mesh,
            } = self.env_cube;

            let uniforms = self.envmap_program.uniforms();
            let program = &self.envmap_program;
            let modelview = cam_view;
            program.use_program();
            program.bind_uniform(uniforms.modelview, &modelview);
            unsafe {
                gl::Disable(gl::CULL_FACE);
                gl::DepthFunc(gl::LEQUAL);
                gl::BindVertexArray(buffers.vertex_array.id());
                gl::DrawElements(
                    gl::TRIANGLES,
                    mesh.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
                gl::DepthFunc(gl::LESS);
            }
        }
        self.draw_entities(scene);
    }
}
