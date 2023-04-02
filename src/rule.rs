use ndarray::Array3;

#[derive(Clone)]
pub enum Neighborhood {
    Moore,
    MooreWrapping,
    VonNeumann,
    VonNeumannWrapping,
}
#[derive(Clone)]
pub struct Rule {
    pub max_state: u8,
    pub survive_mask: [u8; 4],
    pub born_mask: [u8; 4],
    pub neighborhood: Neighborhood,
}

impl Rule {
    pub fn from_closures(
        max_state: u8,
        survive_fn: fn(&u8) -> bool,
        born_fn: fn(&u8) -> bool,
        neighborhood: Neighborhood,
    ) -> Self {
        let mut survive = [0; 4];
        let mut born = [0; 4];
        for i in 0..=27 {
            if survive_fn(&i) {
                survive[(i / 8) as usize] += 1 << (i % 8);
            }
            if born_fn(&i) {
                born[(i / 8) as usize] += 1 << (i % 8);
            }
        }
        Self {
            max_state,
            survive_mask: survive,
            born_mask: born,
            neighborhood,
        }
    }
    pub fn survive(&self, count: u8) -> bool {
        self.survive_mask[(count / 8) as usize] & (1 << (count % 8)) != 0
    }
    pub fn born(&self, count: u8) -> bool {
        self.born_mask[(count / 8) as usize] & (1 << (count % 8)) != 0
    }
    pub fn count_neighbors(
        &self,
        cells: &Array3<u8>,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        match self.neighborhood {
            Neighborhood::Moore => self.moore_neighborhood(
                cells, index,
            ),
            Neighborhood::MooreWrapping => self.moore_neighborhood_wrapping(
                cells, index,
            ),
            Neighborhood::VonNeumann => self.von_neumann_neigborhood(
                cells, index,
            ),
            Neighborhood::VonNeumannWrapping => todo!(),
        }
    }

    fn moore_neighborhood(
        &self,
        cells: &Array3<u8>,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        let dim = cells.dim();
        cells
            .slice(
                ndarray::s![
                    (index.0.checked_sub(1).unwrap_or_default())..(index.0 + 2).min(dim.0),
                    (index.1.checked_sub(1).unwrap_or_default())..(index.1 + 2).min(dim.1),
                    (index.2.checked_sub(1).unwrap_or_default())..(index.2 + 2).min(dim.2)
                ],
            )
            .map(|c| (*c == self.max_state) as u8)
            .sum()
            - ((cells[index] == self.max_state) as u8)
    }
    fn moore_neighborhood_wrapping(
        &self,
        cells: &Array3<u8>,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        let dim = cells.dim();
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
                    ) && cells[new_index] == self.max_state
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
        cells: &Array3<u8>,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        let dim = cells.dim();

        let up = if index.1 + 1 < dim.1 {
            cells[(
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
