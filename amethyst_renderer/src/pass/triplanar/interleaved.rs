//! Simple shaded pass

use std::marker::PhantomData;

use gfx::pso::buffer::ElemStride;
use gfx_core::state::{Blend, ColorMask};

use amethyst_assets::AssetStorage;
use amethyst_core::{
    specs::prelude::{Join, Read, ReadExpect, ReadStorage},
    transform::GlobalTransform,
};

use {
    cam::{ActiveCamera, Camera},
    error::Result,
    hidden::{Hidden, HiddenPropagate},
    light::Light,
    mesh::{Mesh, MeshHandle},
    mtl::{TriplanarMaterial, MaterialDefaults},
    pass::{
        shaded_util::{set_light_args, setup_light_buffers},
        util::{draw_triplanar_mesh, get_camera, setup_triplanar_textures, TriplanarPlane, setup_vertex_args},
    },
    pipe::{
        pass::{Pass, PassData},
        DepthMode, Effect, NewEffect,
    },
    resources::AmbientColor,
    tex::Texture,
    types::{Encoder, Factory},
    vertex::{Normal, Position, Query, TexCoord},
    visibility::Visibility,
};

use super::*;

/// Draw mesh with simple lighting technique
///
/// See the [crate level documentation](index.html) for information about interleaved and separate
/// passes.
///
/// # Type Parameters:
///
/// * `V`: `VertexFormat`
#[derive(Derivative, Clone, Debug, PartialEq)]
#[derivative(Default(bound = "V: Query<(Position, Normal, TexCoord)>"))]
pub struct DrawTriplanar<V> {
    _pd: PhantomData<V>,
    transparency: Option<(ColorMask, Blend, Option<DepthMode>)>,
}

impl<V> DrawTriplanar<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    /// Create instance of `DrawTriplanar` pass
    pub fn new() -> Self {
        Default::default()
    }

    /// Enable transparency
    pub fn with_transparency(
        mut self,
        mask: ColorMask,
        blend: Blend,
        depth: Option<DepthMode>,
    ) -> Self {
        self.transparency = Some((mask, blend, depth));
        self
    }
}

impl<'a, V> PassData<'a> for DrawTriplanar<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    type Data = (
        Option<Read<'a, ActiveCamera>>,
        ReadStorage<'a, Camera>,
        Read<'a, AmbientColor>,
        Read<'a, AssetStorage<Mesh>>,
        Read<'a, AssetStorage<Texture>>,
        ReadExpect<'a, MaterialDefaults>,
        Option<Read<'a, Visibility>>,
        ReadStorage<'a, Hidden>,
        ReadStorage<'a, HiddenPropagate>,
        ReadStorage<'a, MeshHandle>,
        ReadStorage<'a, TriplanarMaterial>,
        ReadStorage<'a, GlobalTransform>,
        ReadStorage<'a, Light>,
    );
}

impl<V> Pass for DrawTriplanar<V>
where
    V: Query<(Position, Normal, TexCoord)>,
{
    fn compile(&mut self, effect: NewEffect) -> Result<Effect> {
        let mut builder = effect.simple(VERT_SRC, FRAG_SRC);
        builder.with_raw_vertex_buffer(V::QUERIED_ATTRIBUTES, V::size() as ElemStride, 0);
        setup_vertex_args(&mut builder);
        setup_light_buffers(&mut builder);
        setup_triplanar_textures(&mut builder, &TEXTURES, TriplanarPlane::PlaneYZ);
        setup_triplanar_textures(&mut builder, &TEXTURES, TriplanarPlane::PlaneXZ);
        setup_triplanar_textures(&mut builder, &TEXTURES, TriplanarPlane::PlaneXY);
        match self.transparency {
            Some((mask, blend, depth)) => builder.with_blended_output("color", mask, blend, depth),
            None => builder.with_output("color", Some(DepthMode::LessEqualWrite)),
        };
        builder.build()
    }

    fn apply<'a, 'b: 'a>(
        &'a mut self,
        encoder: &mut Encoder,
        effect: &mut Effect,
        _factory: Factory,
        (
            active,
            camera,
            ambient,
            mesh_storage,
            tex_storage,
            material_defaults,
            visibility,
            hidden,
            hidden_prop,
            mesh,
            tri_material,
            global,
            light,
        ): <Self as PassData<'a>>::Data,
    ) {
        let camera = get_camera(active, &camera, &global);

        set_light_args(effect, encoder, &light, &global, &ambient, camera);

        match visibility {
            None => {
                for (mesh, tri_material, global, _, _) in
                    (&mesh, &tri_material, &global, !&hidden, !&hidden_prop).join()
                {
                    draw_triplanar_mesh(
                        encoder,
                        effect,
                        false,
                        mesh_storage.get(mesh),
                        None,
                        &tex_storage,
                        Some(tri_material),
                        &material_defaults,
                        camera,
                        Some(global),
                        &[V::QUERIED_ATTRIBUTES],
                        &TEXTURES,
                    );
                }
            }
            Some(ref visibility) => {
                for (mesh, tri_material, global, _) in
                    (&mesh, &tri_material, &global, &visibility.visible_unordered).join()
                {
                    draw_triplanar_mesh(
                        encoder,
                        effect,
                        false,
                        mesh_storage.get(mesh),
                        None,
                        &tex_storage,
                        Some(tri_material),
                        &material_defaults,
                        camera,
                        Some(global),
                        &[V::QUERIED_ATTRIBUTES],
                        &TEXTURES,
                    );
                }

                for entity in &visibility.visible_ordered {
                    if let Some(mesh) = mesh.get(*entity) {
                        draw_triplanar_mesh(
                            encoder,
                            effect,
                            false,
                            mesh_storage.get(mesh),
                            None,
                            &tex_storage,
                            tri_material.get(*entity),
                            &material_defaults,
                            camera,
                            global.get(*entity),
                            &[V::QUERIED_ATTRIBUTES],
                            &TEXTURES,
                        );
                    }
                }
            }
        }
    }
}
