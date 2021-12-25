use priority_queue::PriorityQueue;
use std::cmp::{max, min, Reverse};
use std::collections::HashMap;
use std::io::BufRead;
use std::iter::ExactSizeIterator;

const COSTS: [usize; 5] = [10_000_000, 1, 10, 100, 1000];

const fn format_state(state: u8) -> char {
    match state {
        0 => '.',
        1 => 'A',
        2 => 'B',
        3 => 'C',
        4 => 'D',
        _ => panic!(),
    }
}
const fn room_for_amphipod(state: u8) -> usize {
    (state - 1) as usize
}
const fn room_hallway_pos(room: usize) -> usize {
    2 * (room + 1)
}
const fn hallway_doorway(hallway_pos: usize) -> bool {
    match hallway_pos {
        2 | 4 | 6 | 8 => true,
        _ => false,
    }
}

///
/// ```txt
/// #############
/// #01234567890#
/// ###1#1#1#1###
///   #0#0#0#0#
///   #########
/// ```
///
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct AmphipodBurrow {
    hallway: [u8; 11],
    rooms: [[u8; 2]; 4],
}
impl AmphipodBurrow {
    pub fn is_goal(&self) -> bool {
        0 == self.heuristic()
    }
    pub fn heuristic(&self) -> usize {
        let mut cost = 0;
        // Amphipods in hallway.
        for (start_pos, state) in self.hallway.into_iter().enumerate() {
            if 0 != state {
                let move_cost = COSTS[state as usize];
                let room_index = room_for_amphipod(state);
                cost += move_cost * (max(room_index, start_pos) - min(room_index, start_pos));
                cost += move_cost;
            }
        }
        // Amphipods in rooms.
        for room_index in 0..4 {
            for room_back_front in 0..=1 {
                let state = self.rooms[room_index][room_back_front];
                if 0 != state {
                    let room_target_index = room_for_amphipod(state);
                    if room_target_index != room_index {
                        let move_cost = COSTS[state as usize];
                        cost += move_cost
                            * (max(room_index, room_target_index)
                                - min(room_index, room_target_index));
                        cost += move_cost * 2;
                        cost += move_cost * (1 - room_back_front);
                    }
                }
            }
        }
        cost
    }
    pub fn next_steps(&self, mut visit: impl FnMut(usize, AmphipodBurrow)) {
        // Move from hallway to room.
        'outer: for (start_pos, state) in self.hallway.into_iter().enumerate() {
            if 0 != state {
                // println!("X");
                let move_cost = COSTS[state as usize];
                let room_index = room_for_amphipod(state);
                let room_pos = room_hallway_pos(room_index);

                if 0 != self.rooms[room_index][1] {
                    // println!("A");
                    continue 'outer;
                }

                let mut cost = 0;
                for pos in min(start_pos + 1, room_pos)..=max(start_pos.saturating_sub(1), room_pos) {
                    if 0 != self.hallway[pos] {
                        // println!(
                        //     "B {}->{} blocked by {}",
                        //     min(start_pos, room_pos),
                        //     max(start_pos, room_pos),
                        //     pos
                        // );
                        continue 'outer;
                    }
                    cost += move_cost;
                }
                // Move into back of room.
                if 0 == self.rooms[room_index][0] {
                    let mut next = self.clone();
                    next.hallway[start_pos] = 0;
                    next.rooms[room_index][0] = state;
                    (visit)(cost + 2 * move_cost, next);
                }
                // Move into front of room.
                else if state == self.rooms[room_index][0] {
                    let mut next = self.clone();
                    next.hallway[start_pos] = 0;
                    next.rooms[room_index][1] = state;
                    (visit)(cost + move_cost, next);
                }
            }
        }
        // Move from room to hallway.
        for room_index in 0..4 {
            let room_hw_pos = room_hallway_pos(room_index);

            let (state, room_front_back, start_steps) = if 0 != self.rooms[room_index][1] {
                // Move out of front of room.
                (self.rooms[room_index][1], 1, 1)
            } else if 0 != self.rooms[room_index][0] {
                // Move out of back of room.
                (self.rooms[room_index][0], 0, 2)
            } else {
                continue;
            };

            let target_room_index = room_for_amphipod(state);
            if room_index == target_room_index {
                #[allow(clippy::if_same_then_else)]
                if room_front_back == 0 {
                    // Back already in right spot.
                    continue;
                } else if self.rooms[room_index][0] == state {
                    // Both already in right spot.
                    continue;
                }
            }

            let move_cost = COSTS[state as usize];
            let start_cost = move_cost * start_steps;
            // Go left.
            {
                let mut cum_cost = start_cost;
                for hallway_pos in (0..=room_hw_pos).rev() {
                    if 0 != self.hallway[hallway_pos] {
                        break;
                    }
                    if !hallway_doorway(hallway_pos) {
                        let mut next = self.clone();
                        next.hallway[hallway_pos] = state;
                        next.rooms[room_index][room_front_back] = 0;
                        (visit)(cum_cost, next);
                    }
                    cum_cost += move_cost;
                }
            }
            // Go right.
            {
                let mut cum_cost = start_cost;
                for hallway_pos in room_hw_pos..=10 {
                    if 0 != self.hallway[hallway_pos] {
                        break;
                    }
                    if !hallway_doorway(hallway_pos) {
                        let mut next = self.clone();
                        next.hallway[hallway_pos] = state;
                        next.rooms[room_index][room_front_back] = 0;
                        (visit)(cum_cost, next);
                    }
                    cum_cost += move_cost;
                }
            }
        }
    }
}
impl std::fmt::Display for AmphipodBurrow {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        writeln!(f, "#############")?;
        writeln!(
            f,
            "#{}{}{}{}{}{}{}{}{}{}{}#",
            format_state(self.hallway[0]),
            format_state(self.hallway[1]),
            format_state(self.hallway[2]),
            format_state(self.hallway[3]),
            format_state(self.hallway[4]),
            format_state(self.hallway[5]),
            format_state(self.hallway[6]),
            format_state(self.hallway[7]),
            format_state(self.hallway[8]),
            format_state(self.hallway[9]),
            format_state(self.hallway[10]),
        )?;
        writeln!(
            f,
            "###{}#{}#{}#{}###",
            format_state(self.rooms[0][1]),
            format_state(self.rooms[1][1]),
            format_state(self.rooms[2][1]),
            format_state(self.rooms[3][1])
        )?;
        writeln!(
            f,
            "###{}#{}#{}#{}###",
            format_state(self.rooms[0][0]),
            format_state(self.rooms[1][0]),
            format_state(self.rooms[2][0]),
            format_state(self.rooms[3][0])
        )?;
        writeln!(f, "  #########")?;
        Ok(())
    }
}

fn pathfind_dist(start: AmphipodBurrow) -> Option<usize> {
    let mut dist: HashMap<AmphipodBurrow, usize> = Default::default();
    let mut queue = PriorityQueue::new();
    queue.push(start.clone(), Reverse(0));
    dist.insert(start, 0);

    while let Some((burrow, Reverse(dist_est))) = queue.pop() {
        if burrow.is_goal() {
            return Some(dist_est);
        }

        let d = dist[&burrow];
        burrow.next_steps(|edge_cost, next_burrow| {
            let next_dist = d + edge_cost;
            if dist
                .get(&next_burrow)
                .map(|&old_dist| next_dist < old_dist)
                .unwrap_or(true)
            {
                dist.insert(next_burrow.clone(), next_dist);
                let heuristic = next_burrow.heuristic();
                queue.push(next_burrow, Reverse(next_dist + heuristic));
            }
        });
    }
    None
}

fn main() {
    // let stdin = std::io::stdin();
    // let grid: Vec<Vec<u8>> = stdin
    //     .lock()
    //     .lines()
    //     .map(|line| line.expect("Failed to read line as UTF-8."))
    //     .map(|line| line.into_bytes())
    //     .map(|mut row| {
    //         for b in row.iter_mut() {
    //             *b -= b'0';
    //         }
    //         row
    //     })
    //     .collect();

    // let size = grid.len();
    // assert_eq!(size, grid[0].len());

    // let bound = 5 * size - 1;

    // A: 1
    // B: 2
    // C: 3
    // D: 4

    // Example.
    let start = AmphipodBurrow {
        hallway: Default::default(),
        rooms: [[1, 2], [4, 3], [3, 2], [1, 4]],
    };

    // mingweisamuel
    let start = AmphipodBurrow {
        hallway: Default::default(),
        rooms: [[4, 3], [3, 1], [1, 2], [2, 4]],
    };

    start.next_steps(|cost, next_burrow| {
        println!("{}\n{}", cost, next_burrow);
        if next_burrow.hallway[3] == 2 {
            next_burrow.next_steps(|cost2, next_burrow| {
                println!("{}->{}\n{}", cost, cost2, next_burrow);
                if next_burrow.hallway[5] == 3 && next_burrow.rooms[1][1] == 0 {
                    next_burrow.next_steps(|cost3, next_burrow| {
                        println!("{}->{}->{}\n{}", cost, cost2, cost3, next_burrow);
                    });
                }
            });
        }
    });

    let part_a = pathfind_dist(start).expect("No path found.");
    // let part_a = pathfind_dist(&*grid, size - 1);
    // let part_b = pathfind_dist(&*grid, bound);
    let part_b = part_a;

    println!("{}\n{}", part_a, part_b);
}
