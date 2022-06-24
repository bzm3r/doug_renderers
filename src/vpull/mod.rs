mod pipeline;
mod render_command;
mod render_graph;
mod phase_item;

use bevy::core_pipeline::draw_2d_graph;
use bevy::prelude::*;
use bevy::render::camera::{ActiveCamera, Camera2d, ExtractedCamera};
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_phase::{AddRenderCommand, DrawFunctions, RenderPhase};
use bevy::render::render_resource::{
    BindGroupDescriptor, BindGroupEntry, BufferInitDescriptor, BufferUsages, TextureDescriptor, Extent3d, TextureDimension, TextureFormat, TextureUsages,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};

use bevy::app::{App, Plugin};
use bevy::render::texture::TextureCache;
use bevy::render::view::ViewDepthTexture;
use bevy::render::{RenderApp, RenderStage};
use bevy::utils::HashMap;
use bytemuck::cast_slice;

use crate::gpu_data::{GpuDataBindGroup, GpuPalette, GpuQuad, GpuQuads};
use crate::{BatchedQuads, DRect};

use self::pipeline::{VpullPipeline, QUADS_SHADER_HANDLE};
use self::render_command::DrawQuadsVertexPulling;
use self::render_graph::{VpullPassNode, VPULL_PASS};
use crate::phase_item::VpullPhaseItem;

pub struct VpullPlugin;

impl Plugin for VpullPlugin {
    fn build(&self, app: &mut App) {
        info!("building vertex pull plugin!");
        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            QUADS_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../shaders/vpull.wgsl")),
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<TextureCache>()
            .init_resource::<DrawFunctions<VpullPhaseItem>>()
            .add_render_command::<VpullPhaseItem, DrawQuadsVertexPulling>()
            .init_resource::<VpullPipeline>()
            .init_resource::<GpuQuads>()
            .init_resource::<Palette>()
            .init_resource::<GpuPalette>()
            .add_system_to_stage(RenderStage::Extract, extract_quads_phase)
            .add_system_to_stage(RenderStage::Extract, extract_quads)
            .add_system_to_stage(RenderStage::Prepare, prepare_depth_texture)
            .add_system_to_stage(RenderStage::Prepare, prepare_quads)
            .add_system_to_stage(RenderStage::Queue, queue_quads);

        // connect into the main render graph
        // connect vpull as a node before the main render graph node
        let vpull_pass_node = VpullPassNode::new(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_2d_graph = graph.get_sub_graph_mut(draw_2d_graph::NAME).unwrap();
        draw_2d_graph.add_node(VPULL_PASS, vpull_pass_node);
        draw_2d_graph
            .add_node_edge(VPULL_PASS, draw_2d_graph::node::MAIN_PASS)
            .unwrap();
        draw_2d_graph
            .add_slot_edge(
                draw_2d_graph.input_node().unwrap().id,
                draw_2d_graph::input::VIEW_ENTITY,
                VPULL_PASS,
                VpullPassNode::IN_VIEW,
            )
            .unwrap();
    }
}

#[derive(Clone, Component, Debug, Default)]
struct ExtractedQuads {
    data: Vec<DRect>,
    prepared: bool,
}

struct Palette {
    colors: Vec<Color>,
    prepared: bool,
}

impl Default for Palette {
    fn default() -> Self {
        Palette {
            colors: vec!["648FFF", "785EF0", "DC267F", "FE6100", "FFB000"]
                .into_iter()
                .map(|c| Color::hex(c).unwrap())
                .collect::<Vec<Color>>(),
            prepared: false,
        }
    }
}

// EXTRACT:
// This is the one synchronization point between the Main World and the Render World.
// Relevant Entities, Components, and Resources are read from the Main World and written
// to corresponding Entities, Components, and Resources in the Render World.
// The goal is to keep this step as quick as possible, as it is the one piece of logic
// that cannot run in parallel. It is a good rule of thumb to extract only the minimum
// amount of data needed for rendering, such as by only considering "visible" entities and
// only copying the relevant components.
//
// Entities and components of the render app are cleared every tick, so we must reset them every tick.
fn extract_quads_phase(mut commands: Commands, active_2d: Res<ActiveCamera<Camera2d>>) {
    if let Some(entity) = active_2d.get() {
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<VpullPhaseItem>::default());
    }
}

// The commands in this function are from the Render sub app, but the queries access
// entities from the main app.
fn extract_quads(
    mut commands: Commands,
    mut batched_quads_query: Query<(Entity, &mut BatchedQuads)>,
) {
    for (entity, mut batched_quads) in batched_quads_query.iter_mut() {
        if !batched_quads.extracted {
            let extracted_quads = ExtractedQuads {
                data: batched_quads.data.clone(),
                prepared: false,
            };
            commands
                .get_or_spawn(entity)
                .insert(extracted_quads.clone());
            batched_quads.extracted = true;
            info!("finished extracting quads.");
        } else {
            commands.get_or_spawn(entity).insert(ExtractedQuads {
                data: Vec::new(),
                prepared: true,
            });
        }
    }
}

// PREPARE:
// Extracted data is then "prepared" by writing it to the GPU. This generally involves
// writing to GPU Buffers and Textures and creating Bind Groups.
//
// This time, the resources will come from the render app world.
fn prepare_quads(
    mut commands: Commands,
    mut quads: Query<(Entity, &mut ExtractedQuads)>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut gpu_quads: ResMut<GpuQuads>,
    mut palette: ResMut<Palette>,
    mut gpu_palette: ResMut<GpuPalette>,
    quads_pipeline: Res<VpullPipeline>,
) {
    for (entity, mut quads) in quads.iter_mut() {
        if !quads.prepared {
            quads.prepared = true;
            for quad in quads.data.iter() {
                gpu_quads.instances.push(GpuQuad::from(quad));
            }
            info!("count of rects: {}", gpu_quads.instances.len());
            gpu_quads.index_count = gpu_quads.instances.len() as u32 * 6;
            info!("index count: {}", gpu_quads.index_count);
            let mut indices = Vec::with_capacity(gpu_quads.index_count as usize);
            for i in 0..gpu_quads.instances.len() {
                let base = (i * 4) as u32;
                indices.push(base + 2);
                indices.push(base);
                indices.push(base + 1);
                indices.push(base + 1);
                indices.push(base + 3);
                indices.push(base + 2);
            }
            gpu_quads.index_buffer = Some(render_device.create_buffer_with_data(
                &BufferInitDescriptor {
                    label: Some("gpu_quads_index_buffer"),
                    contents: cast_slice(&indices),
                    usage: BufferUsages::INDEX,
                },
            ));
            gpu_quads
                .instances
                .write_buffer(&*render_device, &*render_queue);
        }

        if !palette.prepared {
            for color in palette.colors.iter() {
                gpu_palette.data.push(color.as_rgba_f32());
            }
            gpu_palette
                .data
                .write_buffer(&*render_device, &*render_queue);
            palette.prepared = true;
        }

        commands
            .get_or_spawn(entity)
            .insert_bundle((GpuDataBindGroup {
                bind_group: render_device.create_bind_group(&BindGroupDescriptor {
                    label: Some("gpu_data_bind_group"),
                    layout: &quads_pipeline.data_layout,
                    entries: &[
                        BindGroupEntry {
                            binding: 0,
                            resource: gpu_quads.instances.buffer().unwrap().as_entire_binding(),
                        },
                        BindGroupEntry {
                            binding: 1,
                            resource: gpu_palette.data.buffer().unwrap().as_entire_binding(),
                        },
                    ],
                }),
            },));
    }
}

pub fn prepare_depth_texture(
    mut commands: Commands,
    mut texture_cache: ResMut<TextureCache>,
    render_device: Res<RenderDevice>,
    views_2d: Query<
        (Entity, &ExtractedCamera), With<RenderPhase<VpullPhaseItem>>,
    >
) {
    let mut textures = HashMap::default();
    for (entity, camera) in views_2d.iter() {
        if let Some(physical_target_size) = camera.physical_size {
            let cached_texture = textures
                .entry(camera.target.clone())
                .or_insert_with(|| {
                    texture_cache.get(
                        &render_device,
                        TextureDescriptor {
                            label: Some("view_depth_texture"),
                            size: Extent3d {
                                depth_or_array_layers: 1,
                                width: physical_target_size.x,
                                height: physical_target_size.y,
                            },
                            mip_level_count: 1,
                            sample_count: 4,
                            dimension: TextureDimension::D2,
                            format: TextureFormat::Depth32Float, /* PERF: vulkan docs recommend using 24
                                                                  * bit depth for better performance */
                            usage: TextureUsages::RENDER_ATTACHMENT,
                        },
                    )
                })
                .clone();
            commands.entity(entity).insert(ViewDepthTexture {
                texture: cached_texture.texture,
                view: cached_texture.default_view,
            });
        }
    }
}

// QUEUE:
// This "queues" render jobs that feed off of "prepared" data.
fn queue_quads(
    opaque_2d_draw_functions: Res<DrawFunctions<VpullPhaseItem>>,
    mut views: Query<&mut RenderPhase<VpullPhaseItem>>,
    quads_query: Query<Entity, With<ExtractedQuads>>,
) {
    let draw_quads = opaque_2d_draw_functions
        .read()
        .get_id::<DrawQuadsVertexPulling>()
        .unwrap();

    for mut opaque_phase in views.iter_mut() {
        for entity in quads_query.iter() {
            opaque_phase.add(VpullPhaseItem {
                entity,
                draw_function: draw_quads,
            });
        }
    }
}
