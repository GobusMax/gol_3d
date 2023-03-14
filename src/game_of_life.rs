use ndarray::{s, Array3};
use ndarray_rand::RandomExt;

#[derive(Clone)]
pub struct GameOfLife {
    pub cells: Array3<u8>,
}

impl GameOfLife {
    pub const MAX_STATE: u8 = 1;
    pub fn new_random(size: usize) -> Self {
        let mut cells = Array3::<u8>::random(
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
        Self { cells }
    }
    pub fn new_single(size: usize) -> Self {
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
        Self { cells }
    }

    pub fn update(&mut self) {
        let old = self.clone();
        for (i, c) in self.cells.indexed_iter_mut() {
            let count = old.moore_neighborhood_wrapping(i);

            if *c == 1 && (13..=26).contains(&count) {
            } else if *c == 0 && ((13..=14).contains(&count) || (17..=19).contains(&count)) {
                *c = Self::MAX_STATE
            } else {
                *c = c.saturating_sub(1);
            }
        }
    }
    fn moore_neighborhood(
        &self,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        let dim = self.cells.dim();
        self.cells
            .slice(
                s![
                    (index.0.checked_sub(1).unwrap_or_default())..(index.0 + 2).min(dim.0),
                    (index.1.checked_sub(1).unwrap_or_default())..(index.1 + 2).min(dim.1),
                    (index.2.checked_sub(1).unwrap_or_default())..(index.2 + 2).min(dim.2)
                ],
            )
            .map(|c| (*c == Self::MAX_STATE) as u8)
            .sum()
            - ((self.cells[index] == Self::MAX_STATE) as u8)
    }
    fn moore_neighborhood_wrapping(
        &self,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        let dim = self.cells.dim();
        let mut sum = 0;
        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let new_index = (
                        (index.0 + dim.0).wrapping_add_signed(x) % dim.0,
                        (index.1 + dim.1).wrapping_add_signed(y) % dim.1,
                        (index.2 + dim.2).wrapping_add_signed(z) % dim.2,
                    );
                    if (
                        x, y, z,
                    ) != (
                        0, 0, 0,
                    ) && self.cells[new_index] == Self::MAX_STATE
                    {
                        sum += 1;
                    }
                }
            }
        }
        sum
    }
    // ! TODO
    fn von_neumann_neigborhood(
        &self,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        let dim = self.cells.dim();

        let up = if index.1 + 1 < dim.1 {
            self.cells[(
                index.0,
                index.1 + 1,
                index.2,
            )]
        } else {
            0
        };
        todo!()
    }
}
