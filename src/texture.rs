use std::num::NonZeroU32;

use anyhow::*;
use image::{DynamicImage, GenericImageView};
use wgpu::*;
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = TextureFormat::Depth32Float;

    pub fn create_depth_texture(device: &Device, config: &SurfaceConfiguration) -> Self {
        let size = Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &TextureDescriptor {
                label: Some("Depth Texture"),
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: Self::DEPTH_FORMAT,
                usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &SamplerDescriptor {
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Linear,
                mipmap_filter: FilterMode::Nearest,
                lod_min_clamp: 0.0,
                lod_max_clamp: 100.,
                compare: Some(CompareFunction::LessEqual),
                ..Default::default()
            },
        );

        Self {
            texture,
            view,
            sampler,
        }
    }

    pub fn _from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<(
        Self,
        BindGroupLayout,
        BindGroup,
    )> {
        let img = image::load_from_memory(bytes)?;
        Ok(
            Self::_from_image(
                device,
                queue,
                &img,
                Some(label),
            ),
        )
    }
    pub fn _from_image(
        device: &Device,
        queue: &Queue,
        image: &DynamicImage,
        label: Option<&str>,
    ) -> (
        Self,
        BindGroupLayout,
        BindGroup,
    ) {
        let rgba = image.to_rgba8();
        let dimensions = image.dimensions();
        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(
            &TextureDescriptor {
                label,
                size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                view_formats: &[],
            },
        );
        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            &rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: NonZeroU32::new(4 * dimensions.0),
                rows_per_image: NonZeroU32::new(dimensions.1),
            },
            size,
        );
        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(
            &wgpu::SamplerDescriptor {
                label: Some("Sampler Descriptor"),
                address_mode_u: AddressMode::ClampToEdge,
                address_mode_v: AddressMode::ClampToEdge,
                address_mode_w: AddressMode::ClampToEdge,
                mag_filter: FilterMode::Linear,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                ..Default::default()
            },
        );
        let bind_group_layout = device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("Texture Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
            },
        );
        let bind_group = device.create_bind_group(
            &wgpu::BindGroupDescriptor {
                label: Some("Diffuse Bind Group"),
                layout: &bind_group_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&sampler),
                    },
                ],
            },
        );

        (
            Self {
                texture,
                view,
                sampler,
            },
            bind_group_layout,
            bind_group,
        )
    }
}
