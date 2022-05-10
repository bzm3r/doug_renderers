use bevy::{
    ecs::system::{
        lifetimeless::{Read, SQuery, SRes},
        SystemParamItem,
    },
    pbr::SetShadowViewBindGroup,
    prelude::Entity,
    render::{
        render_phase::{
            EntityRenderCommand, PhaseItem, RenderCommand, RenderCommandResult, TrackedRenderPass,
        },
        render_resource::{IndexFormat, PipelineCache},
    },
};

use crate::gpu_quads::{GpuQuads, GpuQuadsBindGroup};

use super::pipeline::VpullPipeline;

pub type DrawQuadsVertexPulling = (
    SetQuadsPipeline,
    SetShadowViewBindGroup<0>,
    SetGpuQuadsBindGroup<1>,
    DrawVertexPulledQuads,
);

pub struct SetQuadsPipeline;
impl<P: PhaseItem> RenderCommand<P> for SetQuadsPipeline {
    type Param = (SRes<PipelineCache>, SRes<VpullPipeline>);
    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: &P,
        params: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let (pipeline_cache, vpull_pipeline) = params;
        if let Some(pipeline) = pipeline_cache
            .into_inner()
            .get_render_pipeline(vpull_pipeline.pipeline_id)
        {
            pass.set_render_pipeline(pipeline);
            RenderCommandResult::Success
        } else {
            RenderCommandResult::Failure
        }
    }
}

pub struct SetGpuQuadsBindGroup<const I: usize>;
impl<const I: usize> EntityRenderCommand for SetGpuQuadsBindGroup<I> {
    type Param = SRes<GpuQuadsBindGroup>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        item: Entity,
        gpu_quads_bind_group: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let bind_group = gpu_quads_bind_group
            .into_inner()
            .bind_group
            .as_ref()
            .expect("bind group must have been set before this point!");
        pass.set_bind_group(I, bind_group, &[]);

        RenderCommandResult::Success
    }
}

pub struct DrawVertexPulledQuads;
impl EntityRenderCommand for DrawVertexPulledQuads {
    type Param = SRes<GpuQuads>;

    #[inline]
    fn render<'w>(
        _view: Entity,
        _item: Entity,
        gpu_quads: SystemParamItem<'w, '_, Self::Param>,
        pass: &mut TrackedRenderPass<'w>,
    ) -> RenderCommandResult {
        let gpu_quads = gpu_quads.into_inner();
        pass.set_index_buffer(
            gpu_quads.index_buffer.as_ref().unwrap().slice(..),
            0,
            IndexFormat::Uint32,
        );
        pass.draw_indexed(0..gpu_quads.index_count, 0, 0..1);
        RenderCommandResult::Success
    }
}
