use std::num::NonZeroU32;

use bevy::{
    prelude::{
        AlphaMode, AssetServer, Assets, Handle, Image, Material, Mesh, Res, ResMut, Resource,
    },
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::MeshVertexAttribute,
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, AsBindGroup, AsBindGroupError, BindGroupDescriptor, BindGroupEntry,
            BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource,
            BindingType, PreparedBindGroup, SamplerBindingType, SamplerDescriptor, ShaderRef,
            ShaderStages, TextureSampleType, TextureViewDimension, VertexFormat,
        },
        renderer::RenderDevice,
        texture::FallbackImage,
    },
};

use crate::MAX_TEXTURE_COUNT;

#[derive(Debug, Clone, TypeUuid, TypePath)]
#[uuid = "8dd2b424-45a2-4a53-ac29-7ce356b2d5fe"]
pub struct BindlessMaterial {
    textures: Vec<Handle<Image>>,
}

impl AsBindGroup for BindlessMaterial {
    type Data = ();

    fn as_bind_group(
        &self,
        layout: &BindGroupLayout,
        render_device: &RenderDevice,
        image_assets: &RenderAssets<Image>,
        fallback_image: &FallbackImage,
    ) -> Result<PreparedBindGroup<Self::Data>, AsBindGroupError> {
        // retrieve the render resources from handles
        let mut images = vec![];

        let sampler = render_device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            ..Default::default()
        });

        for handle in self.textures.iter().take(MAX_TEXTURE_COUNT) {
            match image_assets.get(handle) {
                Some(image) => {
                    images.push(image);
                }
                None => return Err(AsBindGroupError::RetryNextUpdate),
            }
        }
        let fallback_image = &fallback_image.d2;

        let textures = vec![&fallback_image.texture_view; MAX_TEXTURE_COUNT];

        // convert bevy's resource types to WGPU's references
        let mut textures: Vec<_> = textures.into_iter().map(|texture| &**texture).collect();

        // fill in up to the first `MAX_TEXTURE_COUNT` textures and samplers to the arrays
        for (id, image) in images.into_iter().enumerate() {
            textures[id] = &*image.texture_view;
        }

        let bind_group = render_device.create_bind_group(&BindGroupDescriptor {
            label: "bindless_material_bind_group".into(),
            layout,
            entries: &[
                // todo 这里甚至可以是特别的类型！！
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureViewArray(&textures[..]),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        Ok(PreparedBindGroup {
            bindings: vec![],
            bind_group,
            data: (),
        })
    }

    fn bind_group_layout(render_device: &RenderDevice) -> BindGroupLayout
    where
        Self: Sized,
    {
        render_device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: "bindless_material_layout".into(),
            entries: &[
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
                    // Note: as textures, multiple samplers can also be bound onto one binding slot.
                    // One may need to pay attention to the limit of sampler binding amount on some platforms.
                    // count: NonZeroU32::new(MAX_TEXTURE_COUNT as u32),
                },
            ],
        })
    }
}

impl Material for BindlessMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/mesh_render.wgsl".into()
    }

    fn vertex_shader() -> ShaderRef {
        "shaders/mesh_render.wgsl".into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(1),
            ATTRIBUTE_DATA.at_shader_location(2),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }

    fn alpha_mode(&self) -> AlphaMode {
        AlphaMode::Opaque
    }

    fn depth_bias(&self) -> f32 {
        0.0
    }

    fn prepass_vertex_shader() -> ShaderRef {
        ShaderRef::Default
    }

    fn prepass_fragment_shader() -> ShaderRef {
        ShaderRef::Default
    }
}

pub const ATTRIBUTE_DATA: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Data", 0x696969, VertexFormat::Uint32);

#[derive(Resource)]
pub struct MaterialStorge(pub Handle<BindlessMaterial>);

impl MaterialStorge {
    pub fn init_with_files(
        asset_server: Res<AssetServer>,
        mut materials: ResMut<Assets<BindlessMaterial>>,
        files: Vec<String>,
    ) -> Self {
        let textures: Vec<_> = files
            .iter()
            .map(|path| {
                // 这里是文件生成的规则 0x,xx
                // > 0 表示 用0补齐两位
                println!("加载资源{}", path);
                asset_server.load(path)
            })
            .collect();
        // 这个东西 可以后续的处理！
        let mat = materials.add(BindlessMaterial { textures });
        Self(mat)
    }
}
