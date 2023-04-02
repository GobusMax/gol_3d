use ndarray::Array3;
use ndarray_rand::RandomExt;

use crate::rule::Rule;

pub struct GameOfLife {
    pub cells: Array3<u8>,
    pub rule: Rule,
}

impl GameOfLife {
    #[allow(dead_code)]
    pub fn new_random_full(size: usize, max_state: u8) -> Array3<u8> {
        Array3::<u8>::random(
            (
                size, size, size,
            ),
            ndarray_rand::rand_distr::Uniform::new(
                0, 2,
            ),
        )
        .map(|v| v * max_state)
    }
    pub fn new_random_partial(size: usize, partial_size: usize, max_state: u8) -> Array3<u8> {
        let mut cells = Array3::<u8>::zeros(
            (
                size, size, size,
            ),
        );
        cells
            .slice_mut(
                ndarray::s![
                    ((size - partial_size) / 2)..((size + partial_size) / 2),
                    ((size - partial_size) / 2)..((size + partial_size) / 2),
                    ((size - partial_size) / 2)..((size + partial_size) / 2),
                ],
            )
            .assign(
                &Array3::<u8>::random(
                    (
                        partial_size,
                        partial_size,
                        partial_size,
                    ),
                    ndarray_rand::rand_distr::Uniform::new(
                        0, 2,
                    ),
                )
                .map(|v| v * max_state),
            );
        cells
    }
    #[allow(dead_code)]
    pub fn new_single(size: usize, rule: Rule) -> Self {
        let mut cells = Array3::<u8>::zeros(
            (
                size, size, size,
            ),
        );
        let half = size / 2;
        cells[(
            half, half, half,
        )] = rule.max_state;
        cells[(
            half + 1,
            half,
            half,
        )] = rule.max_state;
        cells[(
            half,
            half + 1,
            half,
        )] = rule.max_state;
        cells[(
            half + 1,
            half + 1,
            half,
        )] = rule.max_state;
        Self { cells, rule }
    }

    pub fn update(&mut self) {
        let old = self.cells.clone();
        for (i, c) in self.cells.indexed_iter_mut() {
            let count = self.rule.count_neighbors(
                &old, i,
            );

            if *c == 1 && self.rule.survive(count) {
            } else if *c == 0 && self.rule.born(count) {
                *c = self.rule.max_state
            } else {
                *c = c.saturating_sub(1);
            }
        }
    }
}
