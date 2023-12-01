use bitvec::array::BitArray;
use bitvec::order::{BitOrder, Lsb0};
use bitvec::slice::BitSlice;
use bitvec::store::BitStore;
use bitvec::vec::BitVec;
use bitvec::BitArr;

use std::fmt::Write;
use std::io::BufRead;
use std::marker::PhantomData;

pub struct Shape {
    width: usize,
    arrangement: BitArray<[u8; 4]>,
}

const SHAPES: &[Shape] = &[
    Shape {
        width: 4,
        arrangement: BitArray {
            _ord: PhantomData::<Lsb0>,
            data: [0b00001111, 0b00000000, 0b00000000, 0b00000000],
        },
    },
    Shape {
        width: 3,
        arrangement: BitArray {
            _ord: PhantomData::<Lsb0>,
            data: [0b00000010, 0b00000111, 0b00000010, 0b00000000],
        },
    },
    Shape {
        width: 3,
        arrangement: BitArray {
            _ord: PhantomData::<Lsb0>,
            data: [0b00000111, 0b00000100, 0b00000100, 0b00000000],
        },
    },
    Shape {
        width: 1,
        arrangement: BitArray {
            _ord: PhantomData::<Lsb0>,
            data: [0b00000001, 0b00000001, 0b00000001, 0b00000001],
        },
    },
    Shape {
        width: 2,
        arrangement: BitArray {
            _ord: PhantomData::<Lsb0>,
            data: [0b00000011, 0b00000011, 0b00000000, 0b00000000],
        },
    },
];

pub fn grid_string(grid: &BitSlice<impl BitStore, impl BitOrder>) -> String {
    assert_eq!(0, grid.len() % 8, "Grid length must be a multiple of 8.");
    let mut out = String::new();
    for i in (0..grid.len()).step_by(8).rev() {
        for c in grid[i..(i + 7)].iter().by_vals() {
            write!(out, "{}", if c { '#' } else { '.' }).unwrap();
        }
        writeln!(out).unwrap();
    }
    out
}

pub fn intersects(
    a: &BitSlice<impl BitStore, impl BitOrder>,
    b: &BitSlice<impl BitStore, impl BitOrder>,
) -> bool {
    let a = a.iter().by_vals();
    let b = b.iter().by_vals();
    a.zip(b).any(|(a, b)| a && b)
}

pub fn can_move(
    grid: &BitSlice<impl BitStore, impl BitOrder>,
    shape: &Shape,
    offset: usize,
) -> bool {
    !intersects(&grid[offset..], &*shape.arrangement)
}

pub fn grid_string_hypothetical(
    grid: &BitSlice<impl BitStore, impl BitOrder>,
    shape: &Shape,
    offset: usize,
) -> String {
    assert_eq!(0, grid.len() % 8, "Grid length must be a multiple of 8.");
    let mut out = String::new();
    for i in (0..grid.len()).step_by(8).rev() {
        for j in i..(i + 7) {
            let shape = j
                .checked_sub(offset)
                .and_then(|i_off| shape.arrangement.get(i_off))
                .map(|bit_ref| *bit_ref)
                .unwrap_or(false);
            write!(
                out,
                "{}",
                if shape {
                    '@'
                } else if grid[j] {
                    '#'
                } else {
                    '.'
                }
            )
            .unwrap();
        }
        // let grid_row = grid[i..(i + 7)].iter().by_vals();
        // let shape_row = grid
        // for c in  {
        //     write!(out, "{}", if c { '#' } else { '.' }).unwrap();
        // }
        writeln!(out).unwrap();
    }
    out
}

pub fn simulate<'a>(
    grid: &mut BitVec<u8>,
    mut grid_hoff: usize,
    shapes: impl Iterator<Item = (usize, &'a Shape)>,
    mut jets: impl Iterator<Item = isize>,
) -> usize {
    const SPAWN_OFFSET: usize = 3 * 8 + 2;
    const SPAWN_PADDING: usize = 4 * 8;

    for (count, shape) in shapes {
        if 0 == count % 1_000 {
            println!("Round {}", count);
        }

        // Offset of first cell in empty row.
        let height_offset = grid.last_one().map(|i| i - i % 8 + 8).unwrap_or(0);

        grid.resize(
            std::cmp::max(
                grid.capacity(),
                height_offset + shape.arrangement.len() + SPAWN_PADDING,
            ),
            false,
        );

        let mut shape_offset = height_offset + SPAWN_OFFSET;
        for jet in jets.by_ref() {
            // println!("{}", grid_string_hypothetical(&*grid, shape, shape_offset));
            let jetted_offset = ((shape_offset as isize) + jet) as usize;
            if jetted_offset % 8 <= 7 - shape.width && can_move(&*grid, shape, jetted_offset) {
                shape_offset = jetted_offset;
            }
            // println!("{}", jet);

            // println!("{}", grid_string_hypothetical(&*grid, shape, shape_offset));
            if 8 <= shape_offset && can_move(&*grid, shape, shape_offset - 8) {
                shape_offset -= 8;
            } else {
                break;
            }
            // println!("V");
        }

        grid[shape_offset..] |= shape.arrangement;

        // Clear if full row (or paired full row).
        {
            let shape_row_offset = shape_offset - shape_offset % 8;
            for test_row_offset in (shape_row_offset..shape_row_offset + 4 * 8).step_by(8) {
                if 8 <= test_row_offset {
                    let row_a = &grid[test_row_offset - 8..test_row_offset - 1];
                    if row_a.all() {
                        grid_hoff += test_row_offset / 8;

                        let extra = grid.last_one().unwrap();
                        if extra < test_row_offset {
                            grid.fill(false);
                        }
                        else {
                            let rows = grid[test_row_offset..grid.last_one().unwrap()].to_owned();
                            grid.fill(false);
                            *grid |= rows;
                        }
                        break;
                    }
                    // Strict greater than, no point of reseting if offset is already 16
                    else if 16 < test_row_offset {
                        let row_b = &grid[test_row_offset - 16..test_row_offset - 9];
                        let mut row_or = BitArray::<_, Lsb0>::new([0_u8]);
                        row_or |= row_a;
                        row_or |= row_b;
                        if row_or[0..7].all() {
                            grid_hoff += test_row_offset / 8 - 2;

                            let rows = grid[test_row_offset - 16..grid.last_one().unwrap()].to_owned();
                            grid.fill(false);
                            *grid |= rows;
                        }
                    }
                }
            }
        }
    }
    grid_hoff
}

pub fn main() {
    let line = std::io::stdin()
        .lock()
        .lines()
        .map(|line| line.expect("Failed to read line as UTF-8."))
        .next()
        .expect("Expected (at least) one line of input.");
    let jets = line
        .as_bytes()
        .iter()
        .map(|&b| match b {
            b'<' => -1_isize,
            b'>' => 1,
            _ => panic!(),
        })
        .collect::<Vec<_>>();
    let mut jets = jets.into_iter().cycle();
    let mut shapes = SHAPES.iter().cycle().enumerate();

    let mut grid = BitVec::<u8>::new();

    // P1
    let grid_offset = 0;
    let grid_hoff = simulate(
        &mut grid,
        grid_offset,
        shapes.by_ref().take(2022),
        jets.by_ref(),
    );
    println!("{}", grid_hoff + grid.last_one().unwrap_or(0) / 8 + 1);

    // println!("{}", grid_string(&*grid));
    // let mut prev_row = 0;
    // for (row, i) in (0..grid.len()).step_by(8).enumerate() {
    //     if grid[i..(i + 7)].all() {
    //         println!("{}: {}", row - prev_row, row);
    //         prev_row = row;
    //     }
    // }
    // panic!();

    // P2
    let grid_hoff = simulate(
        &mut grid,
        grid_hoff,
        shapes.by_ref().take(1_000_000_000_000 - 2022),
        jets.by_ref(),
    );
    println!("{}", grid_hoff + grid.last_one().unwrap() / 8 + 1);
}
