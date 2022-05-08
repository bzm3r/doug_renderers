mod pipeline;
mod render_command;
mod render_graph;

use bevy::core_pipeline::draw_3d_graph;
use bevy::prelude::*;
use bevy::render::camera::{ActiveCamera, Camera3d};
use bevy::render::render_graph::RenderGraph;
use bevy::render::render_phase::{AddRenderCommand, DrawFunctions, RenderPhase};
use bevy::render::render_resource::{
    BindGroupDescriptor, BindGroupEntry, BufferInitDescriptor, BufferUsages,
};
use bevy::render::renderer::{RenderDevice, RenderQueue};

use bevy::app::{App, Plugin};
use bevy::render::{RenderApp, RenderStage};
use bytemuck::cast_slice;

use crate::gpu_quads::{GpuQuad, GpuQuads, GpuQuadsBindGroup};
use crate::phase_item::QuadsPhaseItem;
use crate::BatchedQuads;

use self::pipeline::{VpullPipeline, QUADS_SHADER_HANDLE};
use self::render_command::DrawQuadsVertexPulling;
use self::render_graph::{VpullPassNode, VPULL_PASS};

struct VpullPlugin;

impl Plugin for VpullPlugin {
    fn build(&self, app: &mut App) {
        app.world.resource_mut::<Assets<Shader>>().set_untracked(
            QUADS_SHADER_HANDLE,
            Shader::from_wgsl(include_str!("../shaders/vpull.wgsl")),
        );

        let render_app = app.sub_app_mut(RenderApp);

        render_app
            .init_resource::<DrawFunctions<QuadsPhaseItem>>()
            .add_render_command::<QuadsPhaseItem, DrawQuadsVertexPulling>()
            .init_resource::<VpullPipeline>()
            .init_resource::<GpuQuads>()
            .add_system_to_stage(RenderStage::Extract, extract_quads_phase)
            .add_system_to_stage(RenderStage::Extract, extract_quads)
            .add_system_to_stage(RenderStage::Prepare, prepare_quads)
            .add_system_to_stage(RenderStage::Queue, queue_quads);

        // connect into the main render graph
        let vpull_pass_node = VpullPassNode::new(&mut render_app.world);
        let mut graph = render_app.world.resource_mut::<RenderGraph>();
        let draw_3d_graph = graph.get_sub_graph_mut(draw_3d_graph::NAME).unwrap();
        draw_3d_graph.add_node(VPULL_PASS, vpull_pass_node);
        draw_3d_graph
            .add_node_edge(VPULL_PASS, draw_3d_graph::node::MAIN_PASS)
            .unwrap();
        draw_3d_graph
            .add_slot_edge(
                draw_3d_graph.input_node().unwrap().id,
                draw_3d_graph::input::VIEW_ENTITY,
                VPULL_PASS,
                VpullPassNode::IN_VIEW,
            )
            .unwrap();
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
fn extract_quads_phase(mut commands: Commands, active_3d: Res<ActiveCamera<Camera3d>>) {
    println!("trying to extract 3d camera...");
    if let Some(entity) = active_3d.get() {
        println!("extracted 3d camera!");
        commands
            .get_or_spawn(entity)
            .insert(RenderPhase::<QuadsPhaseItem>::default());
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
            batched_quads.extracted = true;
            commands.get_or_spawn(entity).insert(batched_quads.clone());
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
    mut batched_quads_query: Query<&mut BatchedQuads>,
    device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    vpull_pipeline: Res<VpullPipeline>,
    mut gpu_quads: ResMut<GpuQuads>,
) {
    let mut some_unprepared = false;
    for mut batched_quads in batched_quads_query.iter_mut() {
        if !batched_quads.prepared {
            some_unprepared = true;
            for rect in batched_quads.data.iter() {
                gpu_quads.instances.push(GpuQuad::from(rect));
            }
        }
        batched_quads.prepared = true;
    }
    if some_unprepared {
        // 2 triangles to make a quad, 3 vertices per triangle, 1 index per vertex = 6 indices per rect
        gpu_quads.index_count = gpu_quads.instances.len() as u32 * 6;
        let mut indices = Vec::with_capacity(gpu_quads.index_count as usize);
        // index order: 2 0 1 1 3 2
        // note that: (0 & 1, 0 & 2) -> (0, 0)
        //            (1 & 1, 1 & 2) -> (1, 0)
        //            (2 & 1, 2 & 2) -> (0, 1)
        //            (3 & 1, 3 & 2) -> (1, 1)
        for i in 0..gpu_quads.instances.len() {
            let base = (i * 6) as u32;
            indices.push(base + 2);
            indices.push(base);
            indices.push(base + 1);
            indices.push(base + 1);
            indices.push(base + 3);
            indices.push(base + 2);
        }
        gpu_quads.index_buffer = Some(device.create_buffer_with_data(&BufferInitDescriptor {
            label: Some("gpu_quads_index_buffer"),
            contents: cast_slice(&indices),
            usage: BufferUsages::INDEX,
        }));
        gpu_quads.instances.write_buffer(&*device, &*render_queue);
        commands.spawn_bundle((GpuQuadsBindGroup {
            bind_group: device.create_bind_group(&BindGroupDescriptor {
                label: Some("gpu_quads_bind_group"),
                layout: &vpull_pipeline.quads_layout,
                entries: &[BindGroupEntry {
                    binding: 0,
                    resource: gpu_quads.instances.buffer().unwrap().as_entire_binding(),
                }],
            }),
        },));
    }
}

// QUEUE:
// This "queues" render jobs that feed off of "prepared" data.
fn queue_quads(
    mut commands: Commands,
    opaque_3d_draw_functions: Res<DrawFunctions<QuadsPhaseItem>>,
    // Question: why are there multiple `QuadsPhaseItem`?
    // Question: why are the `QuadsPhaseItems` called `views`?
    mut views: Query<&mut RenderPhase<QuadsPhaseItem>>,
) {
    let draw_quads = opaque_3d_draw_functions
        .read()
        .get_id::<DrawQuadsVertexPulling>()
        .unwrap();

    for mut opaque_phase in views.iter_mut() {
        let entity_builder = commands.spawn_bundle(());
        opaque_phase.add(QuadsPhaseItem {
            entity: entity_builder.id(),
            draw_function: draw_quads,
        });
    }
}

pub struct VertexPullRendererPlugin;

impl Plugin for VertexPullRendererPlugin {
    fn build(&self, _app: &mut App) {}
}
