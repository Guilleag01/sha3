// Rate: 1088
// Capacity: 512

// use std::arch::x86_64::_mm256_xor_epi64;

use std::array;

const RATE_256: usize = 136;
const TOTAL_STATE_SIZE: usize = 200;
const ROUNDS: usize = 24;
const DELIMITER_SUFFIX: u8 = 0x06; // delimiter suffix for sha3

#[derive(Debug)]
pub struct Sha3_256 {
    state: [u8; TOTAL_STATE_SIZE],
}

impl Default for Sha3_256 {
    fn default() -> Self {
        Self {
            state: [0; TOTAL_STATE_SIZE],
        }
    }
}

impl Sha3_256 {
    pub fn absorb(&mut self, input: &[u8]) {
        // Xor input with rate
        let mut remaining = input.len();
        let mut off = 0;
        let mut in_len = 0;
        while remaining > 0 {
            in_len = remaining.min(RATE_256);
            println!("{}", in_len);
            for i in 0..in_len {
                self.state[i] ^= input[i + off];
            }
            off += in_len - 1;
            remaining -= in_len;

            if in_len == RATE_256 {
                keccak_permute(&mut self.state);
                in_len = 0;
            }
        }

        self.state[in_len] ^= DELIMITER_SUFFIX;

        if (DELIMITER_SUFFIX & 0x80) != 0 && in_len == RATE_256 - 1 {
            keccak_permute(&mut self.state);
        }

        self.state[RATE_256 - 1] ^= 0x80;
    }

    pub fn squeeze<const S: usize>(&mut self) -> [u8; S] {
        keccak_permute(&mut self.state);
        let mut res = [0_u8; S];
        let mut out_len;
        let mut remaining = S;
        let mut off = 0;
        while remaining > 0 {
            out_len = remaining.min(RATE_256);
            res[off..off + out_len].copy_from_slice(&self.state[0..out_len]);
            off += out_len;
            remaining -= out_len;

            if out_len > 0 {
                keccak_permute(&mut self.state);
            }
        }
        // *self.state.first_chunk().unwrap()
        res
    }
}

fn keccak_permute(input: &mut [u8; TOTAL_STATE_SIZE]) {
    let (lanes, _) = input.as_chunks_mut::<8>();
    let mut lfsr_state = 0x01_u8;
    for _ in 0..ROUNDS {
        // θ step
        let c: [u64; 5] = array::from_fn(|x| {
            get_lane(lanes, x, 0)
                ^ get_lane(lanes, x, 1)
                ^ get_lane(lanes, x, 2)
                ^ get_lane(lanes, x, 3)
                ^ get_lane(lanes, x, 4)
        });

        let mut d: u64;
        for x in 0..5 {
            d = c[(x + 4) % 5] ^ rol64(c[(x + 1) % 5], 1);
            for y in 0..5 {
                xor_lane(d, lanes, x, y);
            }
        }

        // ρ and π steps
        let (mut x, mut y) = (1, 0);
        let mut current = get_lane(lanes, x, y);
        let mut temp: u64;

        for t in 0..24 {
            let r = ((t + 1) * (t + 2) / 2) % 64;
            let y2 = (2 * x + 3 * y) % 5;
            x = y;
            y = y2;

            temp = get_lane(lanes, x, y);
            set_lane(rol64(current, r), x, y, lanes);
            current = temp;
        }

        // χ step
        let mut temp2 = [0_u64; 5];
        for y in 0..5 {
            for x in 0..5 {
                temp2[x] = get_lane(lanes, x, y);
            }
            for x in 0..5 {
                set_lane(
                    temp2[x] ^ ((!temp2[(x + 1) % 5]) & temp2[(x + 2) % 5]),
                    x,
                    y,
                    lanes,
                );
            }
        }

        // ι step

        for j in 0..7 {
            let bit_pos: usize = (1 << j) - 1;
            if lfsr86540(&mut lfsr_state) {
                xor_lane((1 as u64) << bit_pos, lanes, 0, 0);
            }
        }
    }
}

#[inline]
fn get_lane(lanes: &[[u8; 8]], x: usize, y: usize) -> u64 {
    u64::from_ne_bytes(lanes[x + 5 * y])
}

#[inline]
fn set_lane(lane: u64, x: usize, y: usize, lanes: &mut [[u8; 8]]) {
    lanes[x + 5 * y] = lane.to_ne_bytes();
}

#[inline]
fn rol64(v: u64, off: usize) -> u64 {
    ((v) << off) ^ ((v) >> (64 - off))
}

#[inline]
fn xor_lane(lane: u64, lanes: &mut [[u8; 8]], x: usize, y: usize) {
    set_lane(get_lane(lanes, x, y) ^ lane, x, y, lanes);
}

// Function that computes the linear feedback shift register (LFSR)
// I have absolutely no idea wtf is this shit. Copied from a github repo lol.
fn lfsr86540(lfsr: &mut u8) -> bool {
    let res = (*lfsr & 0x01) != 0;
    if (*lfsr & 0x80) != 0 {
        *lfsr = (*lfsr << 1) ^ 0x71;
    } else {
        *lfsr <<= 1;
    }
    res
}
