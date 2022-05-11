use bevy::core::{Pod, Zeroable};
use bevy::prelude::*;
use bevy::render::render_resource::{BindGroup, Buffer, BufferUsages, BufferVec};

use crate::DRect;

// Data structure that will be sent to the GPU
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct GpuQuad {
    p0: Vec2,
    p1: Vec2,
    z: Vec2,
}

impl From<&DRect> for GpuQuad {
    fn from(rect: &DRect) -> Self {
        Self {
            p0: Vec2::new(rect.p0.x, rect.p0.y),
            p1: Vec2::new(rect.p1.x, rect.p1.y),
            z: Vec2::new(rect.z, 0.0),
        }
    }
}

#[derive(Component)]
pub struct GpuQuads {
    pub index_buffer: Option<Buffer>,
    pub index_count: u32,
    pub instances: BufferVec<GpuQuad>,
}

impl Default for GpuQuads {
    fn default() -> Self {
        Self {
            index_buffer: None,
            index_count: 0,
            instances: BufferVec::<GpuQuad>::new(BufferUsages::STORAGE),
        }
    }
}

#[derive(Component)]
pub struct GpuQuadsBindGroup {
    pub bind_group: BindGroup,
}
