use bevy::prelude::*;
use bevy::render::render_graph::{
    NodeRunError, RenderGraphContext, RunSubGraphError, SlotInfo, SlotType,
};
use bevy::render::render_phase::{DrawFunctions, RenderPhase, TrackedRenderPass};
use bevy::render::render_resource::{
    LoadOp, Operations, RenderPassDescriptor,
};
use bevy::render::view::{ExtractedView, ViewTarget};
use bevy::render::{render_graph, renderer::RenderContext};

use crate::phase_item::QuadsPhaseItem;
use crate::state::RenderState;

pub struct MainNode {
    pub state: RenderState,
}

impl Default for MainNode {
    fn default() -> Self {
        Self {
            state: RenderState::Loading,
        }
    }
}

impl render_graph::Node for MainNode {
    fn update(&mut self, _world: &mut World) {}

    fn run(
        &self,
        _graph: &mut render_graph::RenderGraphContext,
        _render_context: &mut RenderContext,
        _world: &World,
    ) -> Result<(), NodeRunError> {
        Err(NodeRunError::RunSubGraphError(
            RunSubGraphError::MissingSubGraph("error".into()),
        ))
    }
}

pub const VPULL_PASS: &str = "VPULL_PASS";

pub struct VpullPassNode {
    query: QueryState<
        (&'static RenderPhase<QuadsPhaseItem>, &'static ViewTarget),
        With<ExtractedView>,
    >,
}

impl VpullPassNode {
    pub const IN_VIEW: &'static str = "view";

    pub fn new(world: &mut World) -> Self {
        Self {
            query: QueryState::new(world),
        }
    }
}

impl render_graph::Node for VpullPassNode {
    fn input(&self) -> Vec<SlotInfo> {
        vec![SlotInfo::new(VpullPassNode::IN_VIEW, SlotType::Entity)]
    }

    fn update(&mut self, world: &mut World) {
        self.query.update_archetypes(world);
    }

    fn run(
        &self,
        graph: &mut RenderGraphContext,
        render_context: &mut RenderContext,
        world: &World,
    ) -> Result<(), NodeRunError> {
        let view_entity = graph.get_input_entity(Self::IN_VIEW)?;
        let (quads_phase, target) = match self.query.get_manual(world, view_entity) {
            Ok(query) => query,
            Err(_) => return Ok(()), // No window
        };

        #[cfg(feature = "trace")]
        let _main_vpull_pass_span = info_span!("main_vpull_pass").entered();
        let pass_descriptor = RenderPassDescriptor {
            label: Some("main_vpull_pass"),
            // NOTE: The quads pass loads the color
            // buffer as well as writing to it.
            color_attachments: &[target.get_color_attachment(Operations {
                load: LoadOp::Load,
                store: true,
            })],
            depth_stencil_attachment: None,
        };

        let draw_functions = world.resource::<DrawFunctions<QuadsPhaseItem>>();

        let render_pass = render_context
            .command_encoder
            .begin_render_pass(&pass_descriptor);
        let mut draw_functions = draw_functions.write();
        let mut tracked_pass = TrackedRenderPass::new(render_pass);
        for item in &quads_phase.items {
            let draw_function = draw_functions.get_mut(item.draw_function).unwrap();
            draw_function.draw(world, &mut tracked_pass, view_entity, item);
        }

        Ok(())
    }
}
