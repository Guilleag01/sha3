// Rate: 1088
// Capacity: 512

use std::arch::x86_64::*;

use std::array;

use crate::consts::LFSR_LUT;

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
            for i in 0..in_len {
                self.state[i] ^= input[i + off];
            }
            off += in_len;
            remaining -= in_len;

            if in_len == RATE_256 {
                keccak_permute(&mut self.state);
                in_len = 0;
            }
        }

        // for bytes in inputs_u64 {

        // }

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

        res
    }
}

fn keccak_permute(input: &mut [u8; TOTAL_STATE_SIZE]) {
    // let (lanes, _) = input.as_chunks_mut::<8>();
    let (pre, lanes, post) = unsafe { input.align_to_mut::<u64>() };
    assert!(pre.len() == 0);
    assert!(post.len() == 0);
    assert!(lanes.len() == 25);
    let mut lfsr_state = 0x01_u8;
    for _ in 0..ROUNDS {
        // θ step
        let c: [u64; 5] = array::from_fn(|x| {
            get_lane2(lanes, x, 0)
                ^ get_lane2(lanes, x, 1)
                ^ get_lane2(lanes, x, 2)
                ^ get_lane2(lanes, x, 3)
                ^ get_lane2(lanes, x, 4)
        });

        let mut d: u64;

        for x in 0..5 {
            d = c[(x + 4) % 5] ^ rol64(c[(x + 1) % 5], 1);
            // let mut out = [0_u64; 8];
            // unsafe {
            //     let a: __m512i =
            //         _mm512_set_epi64(d as i64, d as i64, d as i64, d as i64, d as i64, 0, 0, 0);

            //     let b: __m512i = _mm512_set_epi64(
            //         get_lane2(lanes, x, 0) as i64,
            //         get_lane2(lanes, x, 1) as i64,
            //         get_lane2(lanes, x, 2) as i64,
            //         get_lane2(lanes, x, 3) as i64,
            //         get_lane2(lanes, x, 4) as i64,
            //         0,
            //         0,
            //         0,
            //     );
            //     let res = _mm512_xor_epi64(a, b);
            //     _mm512_storeu_epi64(out.as_mut_ptr() as *mut i64, res);
            // }
            // for i in 0..5 {
            //     set_lane2(out[i], x, i, lanes);
            // }
            for y in 0..5 {
                xor_lane2(d, lanes, x, y);
            }
        }

        // ρ and π steps
        let (mut x, mut y) = (1, 0);
        let mut current = get_lane2(lanes, x, y);
        let mut temp: u64;

        for t in 0..24 {
            let r = ((t + 1) * (t + 2) / 2) % 64;
            let y2 = (2 * x + 3 * y) % 5;
            x = y;
            y = y2;

            temp = get_lane2(lanes, x, y);
            set_lane2(rol64(current, r), x, y, lanes);
            current = temp;
        }

        // χ step
        for y in 0..5 {
            // let mut temp2 = [0_u64; 5];
            // for x in 0..5 {
            //     temp2[x] = get_lane(lanes, x, y);
            // }
            let temp2: [u64; 5] = array::from_fn(|x| get_lane2(lanes, x, y));
            for x in 0..5 {
                set_lane2(
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
            let (lfsr_out, new_lfsr) = LFSR_LUT[lfsr_state as usize];
            lfsr_state = new_lfsr;
            // if lfsr86540(&mut lfsr_state) {
            //     xor_lane((1 as u64) << bit_pos, lanes, 0, 0);
            // }

            if lfsr_out {
                xor_lane2((1 as u64) << bit_pos, lanes, 0, 0);
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
fn get_lane2(lanes: &[u64], x: usize, y: usize) -> u64 {
    lanes[x + 5 * y]
}

#[inline]
fn set_lane2(lane: u64, x: usize, y: usize, lanes: &mut [u64]) {
    lanes[x + 5 * y] = lane;
}

#[inline]
fn rol64(v: u64, off: usize) -> u64 {
    ((v) << off) ^ ((v) >> (64 - off))
}

#[inline]
fn xor_lane(lane: u64, lanes: &mut [[u8; 8]], x: usize, y: usize) {
    set_lane(get_lane(lanes, x, y) ^ lane, x, y, lanes);
}

#[inline]
fn xor_lane2(lane: u64, lanes: &mut [u64], x: usize, y: usize) {
    set_lane2(get_lane2(lanes, x, y) ^ lane, x, y, lanes);
}

// Function that computes the linear feedback shift register (LFSR)
// I have absolutely no idea wtf is this shit. Copied from a github repo lol.
// SUSCEPTIBLE TO BE CONVERTED INTO A TABLE
// fn lfsr86540(lfsr: &mut u8) -> bool {
//     let res = (*lfsr & 0x01) != 0;
//     if (*lfsr & 0x80) != 0 {
//         *lfsr = (*lfsr << 1) ^ 0x71;
//     } else {
//         *lfsr <<= 1;
//     }
//     res
// }
