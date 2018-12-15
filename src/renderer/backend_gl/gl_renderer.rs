pub use super::{errors::*, program::*};

use super::{gl_materials::*, objects::*, textures::*};
use cgmath::prelude::*;
use cgmath::*;
use core;
use failure;
use game;
use gl;
use renderer::*;
use sdl2::video::Window;
use std::{
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
    path::Path,
    rc::Rc,
    time::Instant,
};

pub fn rectangle_mesh() -> MeshBuilder {
    let mut mb = MeshBuilder::new();
    mb.positions.append(&mut vec![
        [-1.0f32, -1.0f32, 0.0f32],
        [1.0, -1.0, 0.0],
        [1.0, 1.0, 0.0],
        [-1.0, 1.0, 0.0],
    ]);

    mb.normals.append(&mut vec![
        [0.0f32, 0.0f32, 1.0f32],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, 1.0],
    ]);

    mb.uvs.append(&mut vec![
        [0.0f32, 0.0f32],
        [1.0, 0.0],
        [1.0, 1.0],
        [0.0, 1.0],
    ]);

    mb.indices.append(&mut vec![0, 1, 2, 2, 3, 0]);
    mb
}



fn create_scene_shaders() -> Result<(Program, Program), ShaderError> {
    let scene_program = program_from_sources(
        "./assets/shaders/brdf.vert",
        "./assets/shaders/brdf.frag",
    )?;

    let envmap_program = program_from_sources(
        "./assets/shaders/envmap.vert",
        "./assets/shaders/envmap.frag",
    )?;
    Ok((scene_program, envmap_program))
}

/*
 * Opengl renderer implementation
 */

///
/// Stores uniform ids for the main
/// scene shader
#[derive(Clone, Debug)]
pub struct ShaderUniforms {
    pub modelview: i32,
    pub projection: i32,
    pub normal_matrix: i32,
    pub light_positions: i32,
    user_uniforms: HashMap<String, i32>,
}

impl ShaderUniforms {
    pub fn find_locations(&mut self, program: &Program) {
        self.modelview = program.uniform_location("modelview").unwrap_or(-1);
        self.projection = program.uniform_location("projection").unwrap_or(-1);
        self.normal_matrix =
            program.uniform_location("normal_matrix").unwrap_or(-1);
        self.light_positions =
            program.uniform_location("light_positions").unwrap_or(-1);
        for (key, value) in self.user_uniforms.iter_mut() {
            *value = program.uniform_location(key).unwrap_or(-1);
        }
    }
}

impl Default for ShaderUniforms {
    fn default() -> Self {
        ShaderUniforms {
            modelview: -1,
            projection: -1,
            normal_matrix: -1,
            light_positions: -1,
            user_uniforms: HashMap::new(),
        }
    }
}

struct GlMesh {
    mesh: Mesh,
    buffers: MeshBuffers,
}

impl GlMesh {
    fn skybox_mesh() -> Result<Self, failure::Error> {
        use genmesh::*;
        use renderer::Vertex;
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

type ManagedTextureMaterial = material::Material<Rc<RefCell<GlTexture>>>;

pub struct Materials {
    default_material: ManagedTextureMaterial,
    base_material_ubo: MaterialUbo,
    pub table: Vec<ManagedTextureMaterial>,
}

/// the renderer backend for openGL
pub struct GlRenderer {
    camera: RefCell<Camera>,
    scene_program: Program,
    scene_uniforms: ShaderUniforms,
    envmap_program: Program,
    envmap_uniforms: ShaderUniforms,
    sample_mesh: Mesh,
    env_cube: GlMesh,
    buffers: MeshBuffers,
    pub materials: Materials,
    pub textures: HashMap<String, Rc<RefCell<GlTexture>>>,

    recompile_flag: Cell<Option<Instant>>,
}

impl GlRenderer {
    pub fn new(
        window: &Window,
        model: &model::Model,
    ) -> Result<GlRenderer, RendererError> {
        use super::objects::*;
        let mesh = model.meshes[0].mesh.clone();

        let (width, height) = window.size();
        let perspective = PerspectiveFov {
            fovy: Deg(40.0).into(),
            aspect: width as f32 / height as f32,
            near: 0.1,
            far: 1000.0,
        };
        let (scene_program, envmap_program) =
            create_scene_shaders().map_err(RendererError::ShaderError)?;

        let buffers =
            MeshBuffers::new().map_err(|_| RendererError::Lifecycle {
                reason: format!("could not build gl objects for mesh"),
            })?;
        buffers
            .bind_mesh(&mesh)
            .map_err(|_| RendererError::Lifecycle {
                reason: format!("could not bind buffers to mesh"),
            })?;

        let materials = GlRenderer::make_materials().map_err(|_| {
            RendererError::Lifecycle {
                reason: format!("could create material_obo"),
            }
        })?;

        let env_cube =
            GlMesh::skybox_mesh().map_err(|_| RendererError::Lifecycle {
                reason: format!("could create skybox mesh"),
            })?;

        let texture = GlTexture::new().unwrap();
        println!("created texture {:?}", texture);

        let mut renderer = GlRenderer {
            scene_program,
            scene_uniforms: ShaderUniforms::default(),
            envmap_program,
            envmap_uniforms: ShaderUniforms::default(),
            env_cube,
            sample_mesh: mesh,
            buffers,
            materials,
            textures: HashMap::new(),
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
        self.get_uniform_locations();
        let ref ubo = self.materials.base_material_ubo;
        for i in [&self.envmap_program, &self.scene_program].iter() {
            ubo.bind_to_program(i);
        }

        Ok(())
    }

    fn make_materials() -> Result<Materials, ::failure::Error> {
        use super::objects::*;
        use renderer::material::*;

        use failure::Error;
        let base_material: UntexturedMat = base::PLASTIC_RED;

        let default_material = base_material.transform_textures(|_| None);
        let base_material_ubo = MaterialUbo::new().map_err(&Error::from)?;

        Ok(Materials {
            default_material,
            base_material_ubo,
            table: Vec::new(),
        })
    }

    fn get_uniform_locations(&mut self) {
        self.scene_uniforms.find_locations(&self.scene_program);
        self.envmap_uniforms.find_locations(&self.envmap_program);
    }

    pub fn rebuild_program(&mut self) {
        println!(
            "old shader program {:#?} with uniforms {:#?}",
            self.scene_program, self.scene_uniforms
        );

        let (scene, skybox) = match create_scene_shaders() {
            Ok(mut programs) => programs,
            Err(e) => {
                eprintln!("could not rebuild shaders: {}", e);
                return;
            }
        };

        if let Err(e) = self
            .materials
            .base_material_ubo
            .bind_to_program(&self.scene_program)
        {
            eprintln!("failed to bind material {:?}", e);
        }

        self.scene_program = scene;
        self.envmap_program = skybox;

        self.get_uniform_locations();
        self.scene_program.use_program();
        self.scene_program.bind_uniform(
            self.scene_uniforms.projection,
            &self.camera.borrow().projection,
        );
        self.envmap_program.use_program();

        self.envmap_program.bind_uniform(
            self.envmap_uniforms.projection,
            &self.camera.borrow().projection,
        );

        let ref ubo = self.materials.base_material_ubo;
        for i in &[&self.envmap_program, &self.scene_program] {
            ubo.bind_to_program(i);
        }

        println!(
            "build new shader program {:#?} with uniforms {:#?}",
            self.scene_program, self.scene_uniforms
        );
    }

    #[inline]
    pub fn scene_program(&self) -> &Program {
        &self.scene_program
    }

    #[inline]
    pub fn scene_uniforms(&self) -> &ShaderUniforms {
        &self.scene_uniforms
    }

    fn draw_entities(&self, scene: &game::EntityWorld) {
        use game::component::*;
        use math::*;
        use std::ptr;
        let program = &self.scene_program;
        let uniforms = &self.scene_uniforms;
        let buffers = &self.buffers;
        let mesh = &self.sample_mesh;

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
        unsafe {
            let light_pos_ptr = xformed_light_positions.as_ptr();
            gl::Uniform3fv(
                uniforms.light_positions,
                4,
                light_pos_ptr as *const _,
            );
        }

        let mask = ComponentMask::LIVE_ENTITY
            | ComponentMask::TRANSFORM
            | ComponentMask::STATIC_MESH;

        let entities: Vec<_> = scene
            .components
            .enumerate_entities()
            .filter(|(k, v)| v.contains(mask))
            .collect();

        for (id, mask) in entities {
            let material = if mask.contains(ComponentMask::MATERIAL) {
                let mid = scene.components.materials[&id];
                &self.materials.table[mid.0]
            } else {
                &self.materials.default_material
            };
            if let Err(e) =
                self.materials.base_material_ubo.set_material(material)
            {
                eprintln!("error {:?}", e);
            }

            let transform = scene.components.transforms.get(&id).unwrap();
            let model_matrix = Mat4::from(transform.transform);

            let modelview = cam_view * model_matrix;
            let normal_matrix = modelview.invert().unwrap().transpose();
            program.bind_uniform(uniforms.modelview, &modelview);
            program.bind_uniform(uniforms.normal_matrix, &normal_matrix);
            unsafe {
                gl::BindVertexArray(buffers.vertex_array.id());
                gl::DrawElements(
                    gl::TRIANGLES,
                    mesh.indices.len() as i32,
                    gl::UNSIGNED_INT,
                    ptr::null(),
                );
            }
        }
    }
}

impl Renderer for GlRenderer {
    type Texture = GlTexture;

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

    fn set_clear_color(&mut self, color: Color) {
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
            .bind_uniform(self.scene_uniforms.projection, projection);
        self.envmap_program.use_program();
        self.envmap_program
            .bind_uniform(self.envmap_uniforms.projection, projection);

        unsafe {
            gl::Viewport(0, 0, width as i32, height as i32);
        }
    }

    fn on_update(
        &mut self,
        _delta_time: ::std::time::Duration,
        _world: &game::EntityWorld,
    ) {
        if let Some(t) = self.recompile_flag.get() {
            self.rebuild_program();
            self.recompile_flag.set(None);
        }
    }

    fn flag_shader_recompile(&self) {
        if self.recompile_flag.get().is_none() {
            self.recompile_flag.set(Some(Instant::now()))
        }
    }
}

pub trait RenderScene<S> {
    fn render_scene(&self, mesh: &S);
}

impl RenderScene<game::EntityWorld> for GlRenderer {
    fn render_scene(&self, scene: &game::EntityWorld) {
        use std::ptr;

        use math::*;
        let program = self.scene_program();
        let cam_view = scene.main_camera.transform();

        let uniforms = &self.scene_uniforms;

        {
            let GlMesh {
                ref buffers,
                ref mesh,
            } = self.env_cube;

            let ref uniforms = self.envmap_uniforms;
            let ref program = self.envmap_program;
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
