use ndarray::Array3;
use ndarray_rand::RandomExt;

use crate::rule::Rule;

pub struct GameOfLife {
    pub cells: Array3<u8>,
    rule: Rule,
}

impl GameOfLife {
    pub const MAX_STATE: u8 = 1;
    pub fn new_random(size: usize, rule: Rule) -> Self {
        let cells = Array3::<u8>::random(
            (
                size, size, size,
            ),
            ndarray_rand::rand_distr::Uniform::new(
                0, 2,
            ),
        );

        // cells
        //     .slice_mut(
        //         s![
        //             (size * 3 / 8)..(size * 5 / 8),
        //             (size * 3 / 8)..(size * 5 / 8),
        //             (size * 3 / 8)..(size * 5 / 8)
        //         ],
        //     )
        //     .assign(
        //         &Array3::<u8>::random(
        //             (
        //                 size / 4,
        //                 size / 4,
        //                 size / 4,
        //             ),
        //             ndarray_rand::rand_distr::Uniform::new(
        //                 0, 2,
        //             ),
        //         )
        //         .map(|v| v * Self::MAX_STATE),
        //     );
        Self { cells, rule }
    }
    pub fn new_single(size: usize, rule: Rule) -> Self {
        let mut cells = Array3::<u8>::zeros(
            (
                size, size, size,
            ),
        );
        let half = size / 2;
        cells[(
            half, half, half,
        )] = Self::MAX_STATE;
        cells[(
            half + 1,
            half,
            half,
        )] = Self::MAX_STATE;
        cells[(
            half,
            half + 1,
            half,
        )] = Self::MAX_STATE;
        cells[(
            half + 1,
            half + 1,
            half,
        )] = Self::MAX_STATE;
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
