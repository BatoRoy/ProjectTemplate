//! `backdrop-filter: blur(4px)` for the modal scrim (spec §4.3), as an iced
//! shader primitive (wgpu only; the tiny-skia fallback draws the plain scrim).
//!
//! iced renders layers in order into the frame, and a primitive's `render`
//! gets the encoder and the frame's view. With the surface created with
//! `COPY_SRC` (our one-line iced_wgpu patch) the primitive copies the frame
//! drawn so far, blurs it with a separable Gaussian (σ = 4 CSS px, in sRGB
//! space like Skia), and composites the result back over its bounds with the
//! overlay's fade-in opacity. The scrim colour is drawn by the next layer.

use std::time::Instant;

use iced::advanced::graphics::Viewport;
use iced::mouse;
use iced::widget::shader::{self, Pipeline, Primitive};
use iced::{Element, Length, Rectangle};

use crate::style::ease_out;

pub fn backdrop_blur<'a, Message: 'a>(opened: Instant, animate: bool) -> Element<'a, Message> {
    iced::widget::shader(Blur { opened, animate }).width(Length::Fill).height(Length::Fill).into()
}

struct Blur {
    opened: Instant,
    animate: bool,
}

impl<Message> shader::Program<Message> for Blur {
    type State = ();
    type Primitive = BlurPrimitive;

    fn draw(&self, _state: &(), _cursor: mouse::Cursor, _bounds: Rectangle) -> BlurPrimitive {
        let opacity = if self.animate { ease_out((self.opened.elapsed().as_secs_f32() / 0.150).min(1.0)) } else { 1.0 };
        BlurPrimitive { opacity, sigma_css: 4.0 }
    }
}

#[derive(Debug)]
pub struct BlurPrimitive {
    opacity: f32,
    sigma_css: f32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Params {
    texel: [f32; 2],
    dir: [f32; 2],
    sigma: f32,
    opacity: f32,
    _pad: [f32; 2],
}

fn as_bytes(p: &Params) -> &[u8] {
    unsafe { std::slice::from_raw_parts(p as *const Params as *const u8, std::mem::size_of::<Params>()) }
}

pub struct BlurPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    params_h: wgpu::Buffer,
    params_v: wgpu::Buffer,
    format: wgpu::TextureFormat,
    textures: Option<(wgpu::Extent3d, wgpu::Texture, wgpu::Texture, wgpu::BindGroup, wgpu::BindGroup)>,
}

const SHADER: &str = r#"
struct Params { texel: vec2<f32>, dir: vec2<f32>, sigma: f32, opacity: f32, pad: vec2<f32> };
@group(0) @binding(0) var src: texture_2d<f32>;
@group(0) @binding(1) var samp: sampler;
@group(0) @binding(2) var<uniform> p: Params;

struct Out { @builtin(position) pos: vec4<f32>, @location(0) uv: vec2<f32> };

@vertex
fn vs(@builtin(vertex_index) i: u32) -> Out {
    let xy = vec2<f32>(f32((i << 1u) & 2u), f32(i & 2u));
    var o: Out;
    o.pos = vec4<f32>(xy * 2.0 - 1.0, 0.0, 1.0);
    o.uv = vec2<f32>(xy.x, 1.0 - xy.y);
    return o;
}

@fragment
fn fs(in: Out) -> @location(0) vec4<f32> {
    let r = i32(ceil(p.sigma * 3.0));
    var acc = vec4<f32>(0.0);
    var wsum = 0.0;
    for (var k = -r; k <= r; k = k + 1) {
        let x = f32(k);
        let w = exp(-(x * x) / (2.0 * p.sigma * p.sigma));
        acc = acc + w * textureSampleLevel(src, samp, in.uv + p.dir * p.texel * x, 0.0);
        wsum = wsum + w;
    }
    let c = acc / wsum;
    return vec4<f32>(c.rgb, p.opacity);
}
"#;

impl Pipeline for BlurPipeline {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bato backdrop blur"),
            source: wgpu::ShaderSource::Wgsl(SHADER.into()),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("bato backdrop blur"),
            layout: Some(&pl),
            vertex: wgpu::VertexState { module: &module, entry_point: Some("vs"), buffers: &[], compilation_options: Default::default() },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        let mk = || {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: std::mem::size_of::<Params>() as u64,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        Self { pipeline, layout, sampler, params_h: mk(), params_v: mk(), format, textures: None }
    }
}

impl Primitive for BlurPrimitive {
    type Pipeline = BlurPipeline;

    fn prepare(
        &self,
        pipeline: &mut BlurPipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        _bounds: &Rectangle,
        viewport: &Viewport,
    ) {
        let size = viewport.physical_size();
        let extent = wgpu::Extent3d { width: size.width.max(1), height: size.height.max(1), depth_or_array_layers: 1 };
        if pipeline.textures.as_ref().is_none_or(|t| t.0 != extent) {
            let mk = |usage| {
                device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("bato blur"),
                    size: extent,
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: pipeline.format,
                    usage,
                    view_formats: &[],
                })
            };
            let a = mk(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING);
            let b = mk(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING);
            let bg = |tex: &wgpu::Texture, buf: &wgpu::Buffer| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &pipeline.layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&tex.create_view(&Default::default())),
                        },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&pipeline.sampler) },
                        wgpu::BindGroupEntry { binding: 2, resource: buf.as_entire_binding() },
                    ],
                })
            };
            let bg_h = bg(&a, &pipeline.params_h);
            let bg_v = bg(&b, &pipeline.params_v);
            pipeline.textures = Some((extent, a, b, bg_h, bg_v));
        }
        let texel = [1.0 / extent.width as f32, 1.0 / extent.height as f32];
        let sigma = self.sigma_css * viewport.scale_factor();
        queue.write_buffer(&pipeline.params_h, 0, as_bytes(&Params { texel, dir: [1.0, 0.0], sigma, opacity: 1.0, _pad: [0.0; 2] }));
        queue.write_buffer(
            &pipeline.params_v,
            0,
            as_bytes(&Params { texel, dir: [0.0, 1.0], sigma, opacity: self.opacity, _pad: [0.0; 2] }),
        );
    }

    fn render(
        &self,
        pipeline: &BlurPipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip: &Rectangle<u32>,
    ) {
        let Some((extent, a, b, bg_h, bg_v)) = &pipeline.textures else { return };
        let src = target.texture();
        if src.size() != *extent || !src.usage().contains(wgpu::TextureUsages::COPY_SRC) {
            return;
        }
        encoder.copy_texture_to_texture(src.as_image_copy(), a.as_image_copy(), *extent);
        let pass = |encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, bg: &wgpu::BindGroup, scissor: bool| {
            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bato blur"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            if scissor {
                rp.set_scissor_rect(clip.x, clip.y, clip.width, clip.height);
            }
            rp.set_pipeline(&pipeline.pipeline);
            rp.set_bind_group(0, bg, &[]);
            rp.draw(0..3, 0..1);
        };
        let b_view = b.create_view(&Default::default());
        pass(encoder, &b_view, bg_h, false);
        pass(encoder, target, bg_v, true);
    }
}
