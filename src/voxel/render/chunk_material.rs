use std::{process::exit, num::{NonZeroU32, NonZeroU64}};

use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        extract_component::ExtractComponent,
        mesh::MeshVertexAttribute, renderer::RenderDevice, render_asset::RenderAssets,
        render_resource::*,
        texture::FallbackImage, RenderApp,
    },
    // core::{Zeroable, Pod},
};

use bytemuck::{Pod, Zeroable};

use crate::{voxel::material::VoxelMaterialRegistry, BlockTextures};

const MAX_TEXTURE_COUNT: usize = 15;
const MAX_MATERIAL_COUNT: usize = 256;

#[derive(Component, Clone, Default, ExtractComponent)]
/// A marker component for voxel meshes.
pub struct VoxelTerrainMesh;

impl VoxelTerrainMesh {
    pub const ATTRIBUTE_MATERIAL_INDEX: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Material_Index", 69696969, VertexFormat::Uint32);
}

#[derive(ShaderType, Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct GpuVoxelMaterial {
    base_color: [f32; 4],
    flags: u32,
    emissive: [f32; 4],
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
}

#[derive(Asset, TypePath, Debug, Clone)]
pub struct GpuTerrainMaterial {
    // #[uniform(0)]
    // pub render_distance: u32,
    // #[uniform(0)]
    pub materials: Vec<GpuVoxelMaterial>,
    pub textures: Vec<Handle<Image>>,
}

impl Material for GpuTerrainMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    // fn alpha_mode(&self) -> AlphaMode {
    //     AlphaMode::Opaque
    // }

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
            VoxelTerrainMesh::ATTRIBUTE_MATERIAL_INDEX.at_shader_location(3),
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

        let mut voxel_mats = vec![GpuVoxelMaterial::default(); MAX_MATERIAL_COUNT];
        for (mat, out_mat) in self
            .materials
            .iter()
            .zip(voxel_mats.iter_mut())
            .take(MAX_MATERIAL_COUNT) {
            *out_mat = *mat;
        };

        let bind_group = render_device.create_bind_group(
            "gpu_terrain_material_bind_group",
            layout,
            &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&textures[..]),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&fallback_image.sampler),
                },
                BindGroupEntry {
                    binding: 2,
                    resource: BindingResource::Buffer(BufferBinding {
                        buffer: &render_device.create_buffer_with_data(&BufferInitDescriptor {
                            label: "material_buffer".into(),
                            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                            contents: bytemuck::cast_slice(&voxel_mats)
                        }),
                        offset: 0,
                        size: NonZeroU64::new(
                            u64::from(GpuVoxelMaterial::SHADER_SIZE) * MAX_MATERIAL_COUNT as u64,
                        ),
                    }),
                    // resource: BindingResource::BufferArray(
                    //     &[
                    //         BufferBinding {
                    //             buffer: &render_device.create_buffer_with_data(&BufferInitDescriptor {
                    //                 label: "material_buffer".into(),
                    //                 usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                    //                 contents: bytemuck::cast_slice(&voxel_mats[..]),
                    //             }),
                    //             offset: 0,
                    //             size: NonZeroU64::new(
                    //                 u64::from(GpuVoxelMaterial::SHADER_SIZE) * MAX_TEXTURE_COUNT as u64,
                    //             ),
                    //         }
                    //     ],
                    // )
                },
            ],
            // &BindGroupEntries::with_indices((
            //     (0, &textures[..]),
            //     (1, &fallback_image.sampler),
            //     // (2, &materials[..])
            // )),
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
            // @group(1) @binding(2) var<uniform> materials: array<GpuVoxelMaterial, MAX_TEXTURE_SIZE>
            BindGroupLayoutEntry {
                binding: 2,
                visibility: ShaderStages::FRAGMENT,
                ty: BindingType::Buffer {
                    ty: BufferBindingType::Uniform {},
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: NonZeroU32::new(MAX_MATERIAL_COUNT as u32),
            },
        ]
    }
}

fn update_chunk_material_singleton(
    mut commands: Commands,
    mut materials: ResMut<Assets<GpuTerrainMaterial>>,
    chunk_material: ResMut<ChunkMaterialSingleton>,
    voxel_materials: Res<VoxelMaterialRegistry>,
    mut chunk_entities: Query<(Entity, &mut Handle<GpuTerrainMaterial>)>,
    block_assets: Res<BlockTextures>,
) {
    if chunk_material.is_changed() {
        let mut gpu_mats = GpuTerrainMaterial {
            materials: vec![],
            // render_distance: 32,
            textures: vec![],
        };

        voxel_materials
            .iter_mats()
            .enumerate()
            .for_each(|(_, material)| {
                gpu_mats.materials.push(GpuVoxelMaterial {
                    base_color: material.base_color.as_rgba_f32(),
                    flags: material.flags.bits(),
                    emissive: material.emissive.as_rgba_f32(),
                    perceptual_roughness: material.perceptual_roughness,
                    metallic: material.metallic,
                    reflectance: material.reflectance,
                });
                // gpu_mats.materials[index].base_color = material.base_color.as_linear_rgba_f32();
                // gpu_mats.materials[index].flags = material.flags.bits();
                // gpu_mats.materials[index].emissive = material.emissive.as_linear_rgba_f32();
                // gpu_mats.materials[index].perceptual_roughness = material.perceptual_roughness;
                // gpu_mats.materials[index].metallic = material.metallic;
                // gpu_mats.materials[index].reflectance = material.reflectance;
            });

        // @todo: maybe find a better way to handle textures
        gpu_mats.textures.push(block_assets.void.clone());
        gpu_mats.textures.push(block_assets.void.clone()); // air but since it's also empty it's void.
        gpu_mats.textures.push(block_assets.bedrock.clone());
        gpu_mats.textures.push(block_assets.rock.clone());
        gpu_mats.textures.push(block_assets.dirt.clone());
        gpu_mats.textures.push(block_assets.sand.clone());
        gpu_mats.textures.push(block_assets.grass.clone());
        gpu_mats.textures.push(block_assets.snow.clone());
        gpu_mats.textures.push(block_assets.water.clone());
        gpu_mats.textures.push(block_assets.sandstone.clone());
        gpu_mats.textures.push(block_assets.cactus.clone());
        gpu_mats.textures.push(block_assets.wood.clone());
        gpu_mats.textures.push(block_assets.leaves.clone());
        gpu_mats.textures.push(block_assets.pineleaves.clone());
        gpu_mats.textures.push(block_assets.pinewood.clone());

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
        Self(materials.add(GpuTerrainMaterial {
            materials: vec![],
            textures: vec![],
        }))
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
                    .run_if(resource_exists::<BlockTextures>().and_then(resource_changed::<VoxelMaterialRegistry>()))
                    .in_set(ChunkMaterialSet)
                    // .run_if(in_state(AppState::InGame)),
            );
    }
}