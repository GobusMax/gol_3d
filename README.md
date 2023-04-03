# 3D GOL with wgpu

A 3D Game Of Life capable of simulating arbitrary rules. Rendered with [wgpu](https://wgpu.rs/).

## Run

```sh
cargo run --release
```

For usage information:

```sh
cargo run --release -- --help
```

## Rule Format

### Examples

```
0-6/1,3/2/NN
12-26/13-14/2/M
3,5,7,9,11,15,17,19,21,23-24,26/3,6,8-9,11,14-17,19,24/7/M
0b00110011011000101011110111001010/0b00110101010010101101011111010000/2/M
```

### Grammar

```ebnf
Rule ::= SurviveMask "/" BornMask "/" MaxState "/" Neighborhood

SurviveMask   ::= Mask
BornMask      ::= Mask
MaxState      ::= Number
Neighborhood  ::= "M" | "MN" | "N" | "NN"

Mask     ::= BitMask | ListMask
BitMask  ::= "0b" ( "0" | "1" ) { "0" | "1" }
ListMask ::= [ Number | Range ] { "," ( Number | Range ) } [ "," ]

Range  ::= Number "-" Number
Number ::= "0" | "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9"
```

| Neighborhood Code | Neighborhood Kernel        |
| ----------------- | -------------------------- |
| "M"               | Moore & Wrapping           |
| "MN"              | Moore & Non-Wrapping       |
| "N"               | Von Neumann & Wrapping     |
| "NN"              | Von Neumann & Non-Wrapping |

## References

- https://softologyblog.wordpress.com/2019/12/28/3d-cellular-automata-3/