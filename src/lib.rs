#![no_std]
#![deny(unsafe_code)]
#![deny(missing_docs)]
#![deny(warnings)]
//! Random utf8 utility. This crate is `#![no_std]` but requires `alloc`.
//!
//! ### Example
//!
//! ```rust
//! # use rand::SeedableRng;
//! let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
//! let my_str = rand_utf8::rand_utf8(&mut rng, 32);
//! assert_eq!(32, my_str.as_bytes().len());
//! ```

extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::string::String;
use alloc::vec::Vec;

struct CharUtf8Kind {
    buf: [u8; 32],
    chars: VecDeque<char>,
}

impl CharUtf8Kind {
    pub fn new() -> Self {
        Self {
            buf: [0; 32],
            chars: VecDeque::with_capacity(32),
        }
    }

    pub fn next<R: rand::Rng>(&mut self, rng: &mut R) -> char {
        if self.chars.is_empty() {
            rng.fill(&mut self.buf);
            for c in String::from_utf8_lossy(&self.buf).chars() {
                if c as u32 > 0 && c != char::REPLACEMENT_CHARACTER {
                    self.chars.push_back(c);
                }
            }
        }
        match self.chars.pop_front() {
            None => self.next(rng),
            Some(c) => c,
        }
    }
}

struct CharU32Kind;

impl CharU32Kind {
    pub fn new() -> Self {
        Self
    }

    pub fn next<R: rand::Rng>(&mut self, rng: &mut R) -> char {
        loop {
            let c = rng.random_range(1..=0x110000);
            if let Some(c) = char::from_u32(c) {
                return c;
            }
        }
    }
}

/// Generate a valid random unicode string, targeting well distributed
/// utf8 bytes.
/// rand::distributions::DistString produces random u32 code points,
/// but the majority of these code points produce 4 utf8 bytes each of which
/// are > 128, resulting in poor distribution.
/// This function mixes in random valid utf8 bytes < 128 to fix this issue.
/// Bytes may not be very well distributed if len < 8.
pub fn rand_utf8<R: rand::Rng>(rng: &mut R, len: usize) -> Box<str> {
    let mut chars = Vec::with_capacity(len);
    let mut byte_count = 0;
    let mut utf8_kind = CharUtf8Kind::new();
    let mut u32_kind = CharU32Kind::new();

    while byte_count < len {
        let kind = if len - byte_count < 4 {
            // if we're nearing the end, we need the smaller utf8 kind
            0
        } else {
            // 0, 1, 2, and 3 will give us the smaller utf8 kind
            // 4 will give us the larger u32 kind
            rng.random_range(0..=4)
        };

        if kind < 4 {
            // do the smaller utf8 kind generation which tends to make
            // single utf8 bytes < 128
            let c = utf8_kind.next(rng);
            let c_len = c.len_utf8();
            if byte_count + c_len > len {
                continue;
            }
            byte_count += c_len;
            chars.push(c);
        } else {
            // do the larger u32 kind generation which tends to make
            // 4 byte utf8 blocks with all bytes > 128
            let c = u32_kind.next(rng);
            byte_count += c.len_utf8();
            chars.push(c);
        }
    }

    use rand::seq::SliceRandom;
    chars.shuffle(rng);

    String::from_iter(chars.iter()).into_boxed_str()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn correct_size() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);

        for size in 8..100 {
            let s = rand_utf8(&mut rng, size);
            assert_eq!(size, s.as_bytes().len());
        }
    }

    fn validate_distribution(do_assert: bool, distribution: &[u32; 256]) {
        let mut min = distribution[1];
        let mut max = distribution[1];
        let mut avg_tot = 0.0;
        let mut avg_cnt = 0.0;
        let mut score_sum = 0.0;

        for count in distribution.iter() {
            avg_tot += *count as f64;
            avg_cnt += 1.0;

            if *count < min {
                min = *count;
            }
            if *count > max {
                max = *count;
            }
        }

        let avg = avg_tot / avg_cnt;

        libc_print::libc_println!("min: {}, max: {}, mean: {}", min, max, avg,);

        for (i, count) in distribution.iter().enumerate() {
            let count = *count as f64;

            let dif = if count > avg {
                avg / count
            } else {
                count / avg
            };

            score_sum += dif;

            libc_print::libc_println!("{:03}: {:04} {:0.2}", i, count, dif);

            // this is pretty arbitrary, but if we tweak the algorithm,
            // and it breaks the tests, at least we'll have to
            // consciously change this:
            if do_assert {
                if i >= 1 && i <= 191 {
                    assert!(dif > 0.5);
                } else if i >= 194 && i <= 244 {
                    assert!(dif > 0.06);
                }
            }
        }

        libc_print::libc_println!("-- score_sum: {:0.2} --", score_sum);
    }

    fn distribution_test<F>(do_assert: bool, f: F)
    where
        F: FnOnce(usize, usize) -> [u32; 256],
    {
        let distribution = f(1024, 32);
        validate_distribution(do_assert, &distribution);
    }

    #[test]
    fn distribution() {
        libc_print::libc_println!("# rand::distributions::DistString");

        distribution_test(false, |count, len| {
            use rand::distr::SampleString;
            use rand::SeedableRng;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(2);

            let mut distribution = [0_u32; 256];

            for _ in 0..count {
                for b in rand::distr::StandardUniform
                    .sample_string(&mut rng, len)
                    .as_bytes()
                {
                    distribution[*b as usize] += 1;
                }
            }

            distribution
        });

        libc_print::libc_println!("# rand_utf8");

        distribution_test(true, |count, len| {
            use rand::SeedableRng;
            let mut rng = rand::rngs::SmallRng::seed_from_u64(1);

            let mut distribution = [0_u32; 256];

            for _ in 0..count {
                for b in rand_utf8(&mut rng, len).as_bytes() {
                    distribution[*b as usize] += 1;
                }
            }

            distribution
        });
    }
}
