use std::num::NonZeroU32;

use ndarray::Array3;
use ndarray_rand::RandomExt;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutEntry, Buffer,
    BufferUsages, Device, ShaderStages,
};

use crate::{rule::Rule, Init};

pub const SIZE: usize = 100;
pub struct GameOfLife {
    pub cells: Array3<u8>,
    pub rule: Rule,
    pub init : Init,
}

impl GameOfLife {
    pub fn cells_random(
        size: usize,
        partial_size: usize,
        prob: f64,
        max_state: u8,
    ) -> Array3<u8> {
        let mut cells = Array3::<u8>::zeros((size, size, size));
        cells
            .slice_mut(ndarray::s![
                ((size - partial_size) / 2)..((size + partial_size) / 2),
                ((size - partial_size) / 2)..((size + partial_size) / 2),
                ((size - partial_size) / 2)..((size + partial_size) / 2),
            ])
            .assign(
                &Array3::<bool>::random(
                    (partial_size, partial_size, partial_size),
                    ndarray_rand::rand_distr::Bernoulli::new(prob).unwrap(),
                )
                .map(|v| (*v as u8) * max_state),
            );
        cells
    }

    pub fn cells_random_init(max_state: u8, init: &Init) -> Array3<u8> {
        Self::cells_random(
            SIZE,
            init.size,
            init.density,
            max_state,
        )
    }

    pub fn cells_random_preset(max_state: u8) -> Array3<u8> {
        Self::cells_random(SIZE, 2, 1., max_state)
    }

    pub fn update(&mut self) {
        let old = self.cells.clone();
        for (i, c) in self.cells.indexed_iter_mut() {
            let count = self.rule.count_neighbors(&old, i);

            if *c == 1 && self.rule.survive(count) {
            } else if *c == 0 && self.rule.born(count) {
                *c = self.rule.max_state;
            } else {
                *c = c.saturating_sub(1);
            }
        }
    }

    pub fn generate_buffer(
        &self,
        device: &Device,
    ) -> (Buffer, BindGroupLayout, BindGroup) {
        let buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("GOL Buffer"),
            contents: self.cells.as_slice().unwrap(),
            usage: BufferUsages::STORAGE,
        });
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Compute Bind Group Layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage {
                            read_only: false,
                        },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: Some(NonZeroU32::new((SIZE.pow(3)) as u32).unwrap()),
                }],
            });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Compute Bind Groups"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 7,
                resource: buffer.as_entire_binding(),
            }],
        });
        (buffer, bind_group_layout, bind_group)
    }
}
