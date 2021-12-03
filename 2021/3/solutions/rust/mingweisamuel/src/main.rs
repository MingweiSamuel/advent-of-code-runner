use std::io::BufRead;

#[derive(Debug)]
struct BitVec {
    len: u8,
    bits: usize,
}
impl BitVec {
    pub fn new(len: usize, bits: usize) -> Self {
        assert!((len as u32) < usize::BITS);
        Self {
            len: len as u8,
            bits,
        }
    }
    pub fn len(&self) -> usize {
        self.len as usize
    }
    pub fn as_usize(&self) -> usize {
        self.bits
    }
}
impl std::ops::Index<usize> for BitVec {
    type Output = bool;
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.len as usize);
        let bit_idx = self.len - (index as u8) - 1;
        match 0b1 == 0b1 & (self.bits >> bit_idx) {
            true => &true,
            false => &false,
        }
    }
}
impl std::iter::FromIterator<bool> for BitVec {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = bool>,
    {
        let mut len = 0;
        let bits = iter
            .into_iter()
            .inspect(|_| {
                len += 1;
                assert!(len <= (usize::BITS as u8), "BitVec capacity exceeded");
            })
            .fold(0, |bits, bit| (bits << 1) + (bit as usize));
        Self { len, bits }
    }
}

fn common_bit(rows: &[&BitVec], i: usize) -> bool {
    let occ = rows.iter().fold(0, |occ, row| occ + (row[i] as usize));
    rows.len() <= 2 * occ
}
fn filter_bit(rows: Vec<&BitVec>, i: usize, bit: bool) -> Vec<&BitVec> {
    rows.into_iter().filter(|row| bit == row[i]).collect()
}

fn find_rating(mut rows: Vec<&BitVec>, common: bool) -> Option<&BitVec> {
    let width = rows.first()?.len();
    for i in 0..width {
        if rows.len() <= 1 {
            break;
        }
        let cb = common_bit(&*rows, i);
        rows = filter_bit(rows, i, common ^ cb);
    }
    if 1 < rows.len() {
        None
    } else {
        rows.pop()
    }
}

fn main() {
    let rows: Vec<BitVec> = std::io::stdin()
        .lock()
        .lines()
        .map(|line| {
            let line = line.expect("Failed to read line as UTF-8.");
            let bits = usize::from_str_radix(&*line, 2).expect("Failed to parse binary integer.");
            BitVec::new(line.len(), bits)
        })
        .collect();

    let count = rows.len();
    let width = rows.first().expect("Empty input.").len();

    let commons: BitVec = rows
        .iter()
        .fold(vec![0_usize; width], |mut occ, row| {
            for i in 0..width {
                occ[i] += row[i] as usize;
            }
            occ
        })
        .into_iter()
        .map(|occ| count <= 2 * occ)
        .collect();

    let gamma = commons.as_usize();
    let epsilon = ((0b1 << width) - 1) & (!gamma);

    let generator = find_rating(rows.iter().collect(), true)
        .expect("Failed to determine generator rating.")
        .as_usize();
    let scrubber = find_rating(rows.iter().collect(), false)
        .expect("Failed to determine scrubber rating.")
        .as_usize();

    println!("{}\n{}", gamma * epsilon, generator * scrubber);
}
