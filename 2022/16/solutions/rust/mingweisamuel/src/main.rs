use std::{
    collections::{BTreeMap, HashMap},
    io::BufRead,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct BitSet {
    bits: u64,
}
impl BitSet {
    pub fn set_new(mut self, idx: u32) -> Self {
        self.set(idx);
        self
    }
    pub fn set(&mut self, idx: u32) {
        assert!(idx < u64::BITS);
        self.bits |= 1 << idx;
    }
    pub fn get(self, idx: u32) -> bool {
        1 == 1 & (self.bits >> idx)
    }
    pub fn intersects(self, other: Self) -> bool {
        0 != self.bits & other.bits
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct State {
    visited: BitSet,
    loc: u32,
    released: u32,
    time_used: u32,
    second: bool,
}

const SIZE: usize = 64;

fn process_graph(lines: impl IntoIterator<Item = String>) -> (usize, [[u32; SIZE]; SIZE], u32) {
    // Valve AA has flow rate=0; tunnels lead to valves DD, II, BB
    fn valve_hash(valve: &str) -> u16 {
        assert_eq!(2, valve.as_bytes().len());
        ((valve.as_bytes()[1] as u16) << u8::BITS) | (valve.as_bytes()[0] as u16)
    }

    let mut names = HashMap::with_capacity(60);
    let mut rates = [0; SIZE];
    let mut edges = Vec::<(u32, u32)>::with_capacity(120);

    for (i, line) in lines.into_iter().enumerate() {
        let valve_idx = i as u32;

        let line = line.strip_prefix("Valve ").unwrap();
        let (valve, line) = line.split_once(" has flow rate=").unwrap();
        let valve = valve_hash(valve);
        let (rate, line) = line.split_once(';').unwrap();
        let line = line
            .strip_prefix(" tunnels lead to valves ")
            .unwrap_or_else(|| line.strip_prefix(" tunnel leads to valve ").unwrap());
        let rate: u32 = rate.parse().unwrap();
        let neighbors = line.split(", ").map(valve_hash);

        names.insert(valve, valve_idx);
        rates[valve_idx as usize] = rate;
        edges.extend(
            neighbors
                .filter_map(|neighbor| names.get(&neighbor))
                .copied()
                .map(|neighbor| (neighbor, valve_idx)),
        );
    }
    let start = names[&valve_hash("AA")];
    let size = names.len();
    std::mem::drop(names);

    let mut dists = [[u32::MAX; 64]; 64];
    for (i, j) in edges {
        dists[i as usize][j as usize] = 1;
        dists[j as usize][i as usize] = 1;
    }
    for i in 0..size {
        dists[i][i] = 0;
    }
    for k in 0..size {
        for i in 0..size {
            for j in 0..size {
                let new_dist = dists[i][k].saturating_add(dists[k][j]);
                if new_dist < dists[i][j] {
                    dists[i][j] = new_dist;
                    dists[j][i] = new_dist;
                }
            }
        }
    }

    #[cfg(debug_assertions)]
    {
        for i in 0..size {
            for j in 0..size {
                print!(" {:>2}", dists[i][j]);
            }
            println!();
        }
        println!();
    }

    let mut dists_new = [[u32::MAX; 64]; 64];
    let new_old_iter = (0..size)
        .filter(|&valve| start == (valve as u32) || 0 < rates[valve])
        .enumerate();
    for (i_new, i_old) in new_old_iter.clone() {
        for (j_new, j_old) in new_old_iter.clone() {
            dists_new[i_new][j_new] = dists[i_old as usize][j_old as usize];
        }
    }
    for (i_new, i_old) in new_old_iter.clone() {
        dists_new[i_new][i_new] = rates[i_old];
    }
    std::mem::drop(dists);
    let start = new_old_iter
        .clone()
        .find(|&(_new, old)| start == (old as u32))
        .unwrap()
        .0 as u32;
    let new_size = new_old_iter.count();

    (new_size, dists_new, start)
}

pub fn main() {
    let (size, dists_new, start) = process_graph(
        std::io::stdin()
            .lock()
            .lines()
            .map(|line| line.expect("Failed to read line as UTF-8.")),
    );

    #[cfg(debug_assertions)]
    {
        for i in 0..size {
            for j in 0..size {
                print!(" {:>2}", dists_new[i][j]);
            }
            println!();
        }
        println!();
    }

    let p1 = find_p1(size, &dists_new, start);
    let p2 = find_p2(size, &dists_new, start, p1);

    println!("{}\n{}", p1, p2);
}

fn find_p1(size: usize, dists_new: &[[u32; SIZE]; SIZE], start: u32) -> u32 {
    let mut best = 0;
    let mut init = State {
        loc: start,
        ..Default::default()
    };
    init.visited.set(start);
    let mut stack = vec![init];
    while let Some(state) = stack.pop() {
        if best < state.released {
            best = state.released;
        }
        stack.extend(
            (0..size)
                .filter(|&neighbor| !state.visited.get(neighbor as u32))
                .map(|neighbor| {
                    let time_used = state.time_used + dists_new[state.loc as usize][neighbor] + 1;
                    State {
                        visited: state.visited.set_new(neighbor as u32),
                        loc: neighbor as u32,
                        released: state.released
                            + (30_u32.saturating_sub(time_used)) * dists_new[neighbor][neighbor],
                        time_used,
                        second: state.second,
                    }
                })
                .filter(|valve| valve.time_used < 30),
        )
    }
    best
}

fn find_p2(size: usize, dists_new: &[[u32; SIZE]; SIZE], start: u32, best_possible: u32) -> u32 {
    let mut best = 0;
    let mut init = State {
        loc: start,
        ..Default::default()
    };
    init.visited.set(start);
    let mut stack = vec![init];
    while let Some(state) = stack.pop() {
        if best < state.released {
            println!("{} {}: {}", state.second, state.time_used, state.released);
            best = state.released;
        }
        if !state.second {
            if best < best_possible + state.released {
                stack.push(State {
                    visited: state.visited,
                    loc: start,
                    released: state.released,
                    time_used: 0,
                    second: true,
                });
            }
        }
        stack.extend(
            (0..size)
                .filter(|&neighbor| !state.visited.get(neighbor as u32))
                .map(|neighbor| {
                    let time_used = state.time_used + dists_new[state.loc as usize][neighbor] + 1;
                    State {
                        visited: state.visited.set_new(neighbor as u32),
                        loc: neighbor as u32,
                        released: state.released
                            + (26_u32.saturating_sub(time_used)) * dists_new[neighbor][neighbor],
                        time_used,
                        second: state.second,
                    }
                })
                .filter(|valve| valve.time_used < 26),
        );
    }
    best
}
