#![allow(dead_code)]

//! Use the files in `cool_rules` instead! Just load them with the `-f`/`--file`
//! command line argument.

pub mod as_str {
    pub const SHELLS: &str =
        "3,5,7,9,11,15,17,19,21,23-24,26/3,6,8-9,11,14-17,19,24/7/M";

    pub const CRYSTAL_GROWTH: &str = "0-6/1,3/2/NN";

    pub const CLOUDS2: &str = "12-26/13-14/2/M";

    pub const PERIODIC_FUNKY : &str = "0b00110011011000101011110111001010/0b00110101010010101101011111010000/2/M";
}

pub mod as_rule {
    use crate::rule::{Neighborhood, Rule};

    pub const DODEC: Rule = Rule {
        survive_mask: 0b00010000101000110110100001000010,
        born_mask: 0b10100110001110111001000011111000,
        max_state: 1,
        neighborhood: Neighborhood::Moore,
    };
    pub const WAVY_EXPLOSION: Rule = Rule {
        survive_mask: 0b01110101000100101010001010011110,
        born_mask: 0b01001011101111001000101011001000,
        max_state: 4,
        neighborhood: Neighborhood::Moore,
    };
    pub const LABYRINTH_BOX: Rule = Rule {
        survive_mask: 0b01001111100100001010101100100000,
        born_mask: 0b00001111000101000101100000011110,
        max_state: 4,
        neighborhood: Neighborhood::Moore,
    };
    pub const CITY_BUILER: Rule = Rule {
        survive_mask: 0b10111011110010111010111000011110,
        born_mask: 0b01010010000011010101001001110000,
        max_state: 4,
        neighborhood: Neighborhood::Moore,
    }; // ^ restart often
    pub const GLIDER_HEAVEN: Rule = Rule {
        survive_mask: 0b00110011011000101011110111001010,
        born_mask: 0b00110101010010101101011111010000,
        max_state: 2,
        neighborhood: Neighborhood::Moore,
    };
    pub const PERIODIC_FUNKY: Rule = Rule {
        survive_mask: 0b01111000000011000100000001000010,
        born_mask: 0b11101011011001101100000010010000,
        max_state: 4,
        neighborhood: Neighborhood::Moore,
    };
}
