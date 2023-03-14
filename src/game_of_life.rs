use ndarray::{s, Array3};
use ndarray_rand::RandomExt;

#[derive(Clone)]
pub struct GameOfLife {
    pub cells: Array3<u8>,
}

impl GameOfLife {
    pub fn new_random(size: usize) -> Self {
        let cells = Array3::<u8>::random(
            (
                size, size, size,
            ),
            ndarray_rand::rand_distr::Uniform::new(
                0, 2,
            ),
        );
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
        )] = 1;
        cells[(
            half + 1,
            half,
            half,
        )] = 1;
        cells[(
            half - 1,
            half,
            half,
        )] = 1;
        Self { cells }
    }

    pub fn update(&mut self) {
        let old = self.clone();
        for (i, c) in self.cells.indexed_iter_mut() {
            let count = old.moore_neighborhood(i);
            match count.cmp(&1) {
                std::cmp::Ordering::Less => {}
                std::cmp::Ordering::Equal => *c = 1,
                std::cmp::Ordering::Greater => {}
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
            .sum()
            .checked_sub(self.cells[index])
            .unwrap_or_default()
    }
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
