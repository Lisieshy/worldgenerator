use std::{process::exit, num::NonZeroU32};

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        extract_component::ExtractComponent,
        mesh::MeshVertexAttribute, renderer::RenderDevice, render_asset::RenderAssets,
        render_resource::*,
        texture::FallbackImage, RenderApp,
    },
};

use crate::{MyAssets, AppState, voxel::material::VoxelMaterialRegistry};

const MAX_TEXTURE_COUNT: usize = 2;

#[derive(Component, Clone, Default, ExtractComponent)]
/// A marker component for voxel meshes.
pub struct VoxelTerrainMesh;

impl VoxelTerrainMesh {
    pub const ATTRIBUTE_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Data", 69696969, VertexFormat::Uint32);
}

#[derive(ShaderType, Clone, Copy, Debug, Default)]
pub struct GpuVoxelMaterial {
    base_color: Color,
    flags: u32,
    emissive: Color,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    // alpha: f32,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct GpuTerrainMaterial {
    // #[uniform(0)]
    // pub render_distance: u32,
    // #[uniform(0)]
    // pub materials: [GpuVoxelMaterial; 256],
    pub textures: Vec<Handle<Image>>,
}

impl Default for GpuTerrainMaterial {
    fn default() -> Self {
        Self {
            // render_distance: 16,
            // materials: [default(); 256],
            textures: vec![],
        }
    }
}

impl Material for GpuTerrainMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_NORMAL.at_shader_location(1),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            VoxelTerrainMesh::ATTRIBUTE_DATA.at_shader_location(3),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

impl AsBindGroup for GpuTerrainMaterial {
    type Data = ();

    fn as_bind_group(
            &self,
            layout: &BindGroupLayout,
            render_device: &RenderDevice,
            image_assets: &RenderAssets<Image>,
            fallback_image: &FallbackImage,
        ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        let mut images = vec![];
        for handle in self.textures.iter().take(MAX_TEXTURE_COUNT) {
            match image_assets.get(handle) {
                Some(image) => images.push(image),
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }

        let fallback_image = &fallback_image.d2;

        let textures = vec![&fallback_image.texture_view; MAX_TEXTURE_COUNT];

        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();

        for (id, image) in images.into_iter().enumerate() {
            textures[id] = &*image.texture_view;
        }

        let bind_group = render_device.create_bind_group(
            "voxel_terrain_material_textures_bind_group",
            layout,
            &BindGroupEntries::sequential((&textures[..], &fallback_image.sampler)),
        );

        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: (),
        })
    }

    fn unprepared_bind_group(
            &self,
            _: &BindGroupLayout,
            _: &RenderDevice,
            _: &RenderAssets<Image>,
            _: &FallbackImage,
        ) -> Result<UnpreparedBindGroup<Self::Data>, AsBindGroupError> {
        panic!("bindless textures not supported. this shouldn't happen.");
    }

    fn bind_group_layout_entries(_: &RenderDevice) -> Vec<BindGroupLayoutEntry>
        where
            Self: Sized {
        vec![
            // @group(1) @binding(0) var textures: binding_array<texture_2d<f32>>;
            BindGroupLayoutEntry {
                binding: 0,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
                count: NonZeroU32::new(MAX_TEXTURE_COUNT as u32),
            },
            // @group(1) @binding(1) var nearest_sampler: sampler;
            BindGroupLayoutEntry {
                binding: 1,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Sampler(SamplerBindingType::Filtering),
                count: None,
            },
        ]
    }
}

fn update_chunk_material_singleton(
    mut commands: Commands,
    mut materials: ResMut<Assets<GpuTerrainMaterial>>,
    chunk_material: ResMut<ChunkMaterialSingleton>,
    // voxel_materials: Res<VoxelMaterialRegistry>,
    mut chunk_entities: Query<(Entity, &mut Handle<GpuTerrainMaterial>)>,
    assets: Res<MyAssets>,
) {
    if chunk_material.is_changed() {
        let mut gpu_mats = GpuTerrainMaterial {
            // materials: [GpuVoxelMaterial {
            //     base_color: Color::WHITE,
            //     flags: 0,
            //     ..Default::default()
            // }; 256],
            // render_distance: 32,
            textures: vec![],
        };

        // voxel_materials
        //     .iter_mats()
        //     .enumerate()
        //     .for_each(|(index, material)| {
        //         gpu_mats.materials[index].base_color = material.base_color;
        //         gpu_mats.materials[index].flags = material.flags.bits();
        //         gpu_mats.materials[index].emissive = material.emissive;
        //         gpu_mats.materials[index].perceptual_roughness = material.perceptual_roughness;
        //         gpu_mats.materials[index].metallic = material.metallic;
        //         gpu_mats.materials[index].reflectance = material.reflectance;
        //     });

        for (_, texture) in assets.tiles.iter().enumerate() {
            gpu_mats.textures.push(texture.clone());
        }

        let chunk_material = materials.add(gpu_mats);
        commands.insert_resource(ChunkMaterialSingleton(chunk_material.clone()));

        for (_, mut mat) in &mut chunk_entities {
            *mat = chunk_material.clone();
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ChunkMaterialSingleton(Handle<GpuTerrainMaterial>);

impl FromWorld for ChunkMaterialSingleton {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<GpuTerrainMaterial>>();
        Self(materials.add(GpuTerrainMaterial::default()))
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, SystemSet)]
/// Systems that prepare the global [ChunkMaterialSingleton] value.
pub struct ChunkMaterialSet;

pub struct ChunkMaterialPlugin;

struct GpuFeatureSupportChecker;

impl Plugin for GpuFeatureSupportChecker {
    fn build(&self, _app: &mut App) {}

    fn finish(&self, app: &mut App) {
        let Ok(render_app) = app.get_sub_app_mut(RenderApp) else {
            return;
        };

        let render_device = render_app.world.resource::<RenderDevice>();

        if !render_device
            .features()
            .contains(WgpuFeatures::SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING)
        {
            error!(
                "Render device doesn't support feature \
SAMPLED_TEXTURE_AND_STORAGE_BUFFER_ARRAY_NON_UNIFORM_INDEXING, \
which is required for the renderer."
            );
            exit(1);
        }
    }
}

impl Plugin for ChunkMaterialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins((
                GpuFeatureSupportChecker,
                MaterialPlugin::<GpuTerrainMaterial>::default()
            ))
            .init_resource::<ChunkMaterialSingleton>()
            .add_systems(
                Update,
                update_chunk_material_singleton
                    .run_if(resource_changed::<VoxelMaterialRegistry>()) // @todo: not this way it's ugly
                    .in_set(ChunkMaterialSet)
                    .run_if(in_state(AppState::InGame)),
            );
    }
}