use bevy::core::{Pod, Zeroable};
use bevy::prelude::*;
use bevy::render::render_resource::{BindGroup, Buffer, BufferUsages, BufferVec};

use crate::DRect;

// Data structure that will be sent to the GPU
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
#[repr(C)]
pub struct GpuQuad {
    pub p0: Vec2,
    pub p1: Vec2,
    pub layer: f32,
    pub color: u32,
}

impl From<&DRect> for GpuQuad {
    fn from(rect: &DRect) -> Self {
        Self {
            p0: Vec2::new(rect.p0.x, rect.p0.y),
            p1: Vec2::new(rect.p1.x, rect.p1.y),
            layer: rect.layer,
            color: rect.color,
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
pub struct GpuDataBindGroup {
    pub bind_group: BindGroup,
}

#[derive(Component)]
pub struct GpuPalette {
    pub data: BufferVec<[f32; 4]>,
}

impl Default for GpuPalette {
    fn default() -> Self {
        let mut result = Self {
            data: BufferVec::<[f32; 4]>::new(BufferUsages::STORAGE),
        };
        result.data.push([1.0, 1.0, 1.0, 1.0]);
        result
    }
}
