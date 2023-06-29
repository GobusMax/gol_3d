use wgpu::{
    Device, Extent3d, SurfaceConfiguration, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureView,
    TextureViewDescriptor,
};
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
}

impl Texture {
    pub const DEPTH_FORMAT: wgpu::TextureFormat = TextureFormat::Depth32Float;

    pub fn create_depth_texture(
        device: &Device,
        config: &SurfaceConfiguration,
    ) -> Self {
        let size = Extent3d {
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Depth Texture"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: Self::DEPTH_FORMAT,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        Self { texture, view }
    }
}
