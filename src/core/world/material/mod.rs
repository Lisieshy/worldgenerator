use std::{process::exit, num::{NonZeroU32, NonZeroU64}};

use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        extract_component::ExtractComponent,
        mesh::MeshVertexAttribute, renderer::RenderDevice, render_asset::RenderAssets,
        render_resource::*,
        texture::FallbackImage, RenderApp,
    },
};

use bytemuck::{Pod, Zeroable};

pub(super) fn setup(app: &mut App) {
    app
        .add_plugins(GpuFeatureSupportChecker)
        .add_plugins(MaterialPlugin::<BlockMaterial>::default());
}

const MAX_TEXTURE_COUNT: usize = 48;
const MAX_MATERIAL_COUNT: usize = 16;

#[derive(Component, Clone, Default, ExtractComponent)]
/// A marker component for voxel meshes.
pub struct VoxelTerrainMesh;

impl VoxelTerrainMesh {
    pub const ATTRIBUTE_TEXTURE_INDEX: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Texture_Index", 998875654, VertexFormat::Uint32);

    pub const ATTRIBUTE_MATERIAL_INDEX: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Material_Index", 994485651, VertexFormat::Uint32);

}

#[derive(ShaderType, Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct VoxelMaterial {
    emissive_r: f32,
    emissive_g: f32,
    emissive_b: f32,
    emissive_a: f32,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
}

#[derive(Asset, Default, TypePath, Debug, Clone, TypeUuid)]
#[uuid = "01159b10-d6b1-4b16-8579-c4804e7be96b"]
pub struct BlockMaterial {
    pub materials: Vec<VoxelMaterial>,
    pub textures: Vec<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl BlockMaterial {
    pub fn new_opaque(
        materials: Vec<VoxelMaterial>,
        textures: Vec<Handle<Image>>
    ) -> Self {
        Self {
            materials,
            textures,
            alpha_mode: AlphaMode::Opaque,
        }
    }

    pub fn new_blended(
        materials: Vec<VoxelMaterial>,
        textures: Vec<Handle<Image>>
    ) -> Self {
        Self {
            materials,
            textures,
            alpha_mode: AlphaMode::Blend,
        }
    }
}

impl Material for BlockMaterial {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
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
            VoxelTerrainMesh::ATTRIBUTE_TEXTURE_INDEX.at_shader_location(3),
            VoxelTerrainMesh::ATTRIBUTE_MATERIAL_INDEX.at_shader_location(4),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

impl AsBindGroup for BlockMaterial {
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

        let mut voxel_mats = vec![VoxelMaterial::default(); MAX_MATERIAL_COUNT];
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
                            u64::from(VoxelMaterial::SHADER_SIZE) * MAX_MATERIAL_COUNT as u64,
                        ),
                    }),
                },
            ],
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
