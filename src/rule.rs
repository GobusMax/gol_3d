use std::{ops::RangeInclusive, str::FromStr, fmt::Display};

use ndarray::Array3;
use ndarray_rand::rand;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Neighborhood {
    Moore,
    MooreWrapping,
    VonNeumann,
    VonNeumannWrapping,
}

pub struct Rule {
    pub survive_mask: u32,
    pub born_mask: u32,
    pub max_state: u8,
    pub neighborhood: Neighborhood,
}

impl Rule {
    #[allow(dead_code)]
    pub fn new<T: ToBitMask, U: ToBitMask>(
        survive: T,
        born: U,
        max_state: u8,
        neighborhood: Neighborhood,
    ) -> Self {
        Self {
            survive_mask: survive.to_bit_mask(),
            born_mask: born.to_bit_mask(),
            max_state,
            neighborhood,
        }
    }
    pub fn new_random() -> Self{

        Self { survive_mask: rand::random::<u32>() & (u32::MAX-1), born_mask:rand::random::<u32>() & (u32::MAX-1), max_state:rand::random::<u8>()/64 + 1, neighborhood: Neighborhood::MooreWrapping }
    }
    pub fn survive(&self, count: u8) -> bool {
        self.survive_mask & (1 << count) != 0
    }
    pub fn born(&self, count: u8) -> bool {
        self.born_mask & (1 << count) != 0
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
            Neighborhood::VonNeumann => self.von_neumann_neigborhood2(
                cells, index,
            ),
            Neighborhood::VonNeumannWrapping => todo!(),
        }
    }

    fn _moore_neighborhood_old(
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
        let mut sum = 0;
        for x in -1..=1 {
            if index.0.checked_add_signed(x).unwrap_or(dim.0) < dim.0 {
                for y in -1..=1 {
                    if index.1.checked_add_signed(y).unwrap_or(dim.1) < dim.1 {
                        for z in -1..=1 {
                            if index.2.checked_add_signed(z).unwrap_or(dim.2) < dim.2 {
                                let new_index = (
                                    (index.0).wrapping_add_signed(x),
                                    (index.1).wrapping_add_signed(y),
                                    (index.2).wrapping_add_signed(z),
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
                }
            }
        }
        sum
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
    #[rustfmt::skip]
    fn von_neumann_neigborhood2(
        &self,
        cells: &Array3<u8>,
        index: (
            usize,
            usize,
            usize,
        ),
    ) -> u8 {
        let dim = cells.dim();

          ((index.0 + 1 < dim.0 && cells[(index.0 + 1,index.1,index.2)] == self.max_state) as u8)
        + ((index.1 + 1 < dim.1 && cells[(index.0,index.1 + 1,index.2)] == self.max_state) as u8)
        + ((index.2 + 1 < dim.2 && cells[(index.0,index.1,index.2 + 1)] == self.max_state) as u8)
        + ((index.0 > 0         && cells[(index.0 - 1,index.1,index.2)] == self.max_state) as u8)
        + ((index.1 > 0         && cells[(index.0,index.1 - 1,index.2)] == self.max_state) as u8)
        + ((index.2 > 0         && cells[(index.0,index.1,index.2 - 1)] == self.max_state) as u8)
    }
    #[rustfmt::skip]
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
        let mut sum = 0;
        
        if index.0 + 1 < dim.0 && cells[(index.0 + 1,index.1,index.2)] == self.max_state { sum += 1}
        if index.1 + 1 < dim.1 && cells[(index.0,index.1 + 1,index.2)] == self.max_state { sum += 1}
        if index.2 + 1 < dim.2 && cells[(index.0,index.1,index.2 + 1)] == self.max_state { sum += 1}
        if index.0 > 0         && cells[(index.0 - 1,index.1,index.2)] == self.max_state { sum += 1}
        if index.1 > 0         && cells[(index.0,index.1 - 1,index.2)] == self.max_state { sum += 1}
        if index.2 > 0         && cells[(index.0,index.1,index.2 - 1)] == self.max_state { sum += 1}

        sum
    }
}
impl Display for Rule{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rule{{\nsurvive_mask: {:#034b},\nborn_mask:    {:#034b},\nmax_state: {},\nneighborhood: rule::Neighborhood::{:?}\n}};",self.survive_mask,self.born_mask,self.max_state,self.neighborhood)
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct ParseRuleError;
impl FromStr for Rule {
    type Err = ParseRuleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        let survive_mask = parts[0].to_bit_mask();
        let born_mask = parts[1].to_bit_mask();

        let max_state = parts[2].parse::<u8>().unwrap() - 1;
        let neighborhood: Neighborhood = match parts[3] {
            "M" => Neighborhood::MooreWrapping,
            "MNW" => Neighborhood::Moore,
            "N" => Neighborhood::VonNeumannWrapping,
            "NNW" => Neighborhood::VonNeumann,
            _ => return Err(ParseRuleError),
        };
        Ok(
            Self {
                survive_mask,
                born_mask,
                max_state,
                neighborhood,
            },
        )
    }
}

pub trait ToBitMask {
    fn to_bit_mask(self) -> u32;
}
impl ToBitMask for RangeInclusive<u8> {
    fn to_bit_mask(self) -> u32 {
        let mut mask = 0;
        for i in self {
            mask |= 1 << i;
        }
        mask
    }
}
impl<F> ToBitMask for F
where
    F: Fn(u8) -> bool,
{
    fn to_bit_mask(self) -> u32 {
        let mut mask = 0;
        for i in 0..=27 {
            if self(i) {
                mask |= 1 << i;
            }
        }
        mask
    }
}
impl ToBitMask for u8 {
    fn to_bit_mask(self) -> u32 {
        1 << self
    }
}
impl ToBitMask for &str {
    fn to_bit_mask(self) -> u32 {
        let mut mask = 0;
        for p in self.split(',') {
            if p.contains('-') {
                let mut split = p.split('-');
                let range = (split.next().unwrap().parse::<u8>().unwrap()
                    ..=split.next().unwrap().parse::<u8>().unwrap())
                    .to_bit_mask();
                mask |= range;
            } else if !p.is_empty() {
                mask |= 1 << p.parse::<u8>().unwrap();
            }
        }
        mask
    }
}
