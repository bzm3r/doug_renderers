use bevy::{
    prelude::Entity,
    render::render_phase::{DrawFunctionId, EntityPhaseItem, PhaseItem},
};

pub struct VpullPhaseItem {
    pub entity: Entity,
    pub draw_function: DrawFunctionId,
}

impl PhaseItem for VpullPhaseItem {
    // The type used for ordering the items. The smallest values are drawn first.
    type SortKey = u32;

    #[inline]
    fn sort_key(&self) -> Self::SortKey {
        0
    }

    #[inline]
    fn draw_function(&self) -> DrawFunctionId {
        self.draw_function
    }
}

impl EntityPhaseItem for VpullPhaseItem {
    #[inline]
    fn entity(&self) -> Entity {
        self.entity
    }
}
