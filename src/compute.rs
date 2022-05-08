// use bevy::core_pipeline::node::MAIN_PASS_DEPENDENCIES;
// use bevy::prelude::World;
// use bevy::render::render_graph::{NodeRunError, RunSubGraphError};
// use bevy::render::renderer::{RenderContext, RenderDevice};
// use bevy::{
//     app::{App, Plugin},
//     render::{render_graph, RenderApp},
// };

// use crate::state::RenderState;

// pub struct ComputeRendererPlugin;

// impl Plugin for ComputeRendererPlugin {
//     fn build(&self, app: &mut App) {}
// }

// pub struct MainNode {
//     pub state: RenderState,
// }

// impl Default for MainNode {
//     fn default() -> Self {
//         Self {
//             state: RenderState::Loading,
//         }
//     }
// }

// impl render_graph::Node for MainNode {
//     fn update(&mut self, world: &mut World) {}

//     fn run(
//         &self,
//         _graph: &mut render_graph::RenderGraphContext,
//         render_context: &mut RenderContext,
//         world: &World,
//     ) -> Result<(), NodeRunError> {
//         Err(NodeRunError::RunSubGraphError(
//             RunSubGraphError::MissingSubGraph("error".into()),
//         ))
//     }
// }
