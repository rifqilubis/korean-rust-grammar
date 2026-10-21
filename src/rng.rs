//! PRNG kecil (xorshift64\*) supaya pengacakan soal tidak butuh crate eksternal.
//!
//! Bukan untuk keperluan kriptografi — hanya untuk mengocok urutan soal.

/// Generator xorshift64\* yang deterministik bila diberi seed.
#[derive(Clone, Debug)]
pub struct Rng {
    state: u64,
}

impl Rng {
    /// Buat generator dari seed. Seed 0 diganti agar state tidak pernah nol.
    pub fn new(seed: u64) -> Rng {
        Rng {
            state: if seed == 0 {
                0x9E37_79B9_7F4A_7C15
            } else {
                seed
            },
        }
    }

    /// Seed dari jam sistem (nanodetik) — cukup acak untuk sesi belajar.
    pub fn from_time() -> Rng {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEAD_BEEF_CAFE_1234);
        Rng::new(nanos ^ (std::process::id() as u64) << 32)
    }

    /// Bilangan 64-bit berikutnya.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Bilangan pada rentang `0..bound`. Mengembalikan 0 bila `bound == 0`.
    pub fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            0
        } else {
            (self.next_u64() % bound as u64) as usize
        }
    }

    /// Bilangan pada rentang `min..=max`.
    pub fn range(&mut self, min: usize, max: usize) -> usize {
        if max <= min {
            min
        } else {
            min + self.below(max - min + 1)
        }
    }

    /// Acak urutan isi slice (Fisher–Yates).
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        if items.len() < 2 {
            return;
        }
        for i in (1..items.len()).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }

    /// Ambil `count` elemen acak tanpa pengulangan (bila cukup).
    pub fn sample<T: Clone>(&mut self, items: &[T], count: usize) -> Vec<T> {
        let mut pool: Vec<T> = items.to_vec();
        self.shuffle(&mut pool);
        pool.truncate(count);
        pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_with_same_seed() {
        let a: Vec<u64> = (0..8).map(|_| Rng::new(42).next_u64()).collect();
        let mut r1 = Rng::new(42);
        let mut r2 = Rng::new(42);
        for _ in 0..16 {
            assert_eq!(r1.next_u64(), r2.next_u64());
        }
        assert_eq!(a[0], a[7], "seed sama harus menghasilkan nilai sama");
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = Rng::new(7);
        for _ in 0..1000 {
            let v = r.below(5);
            assert!(v < 5);
        }
        assert_eq!(r.below(0), 0);
    }

    #[test]
    fn shuffle_keeps_elements() {
        let mut r = Rng::new(2024);
        let mut v: Vec<u32> = (0..20).collect();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, (0..20).collect::<Vec<u32>>());
    }

    #[test]
    fn sample_truncates() {
        let mut r = Rng::new(11);
        let items = [1, 2, 3, 4, 5];
        let picked = r.sample(&items, 3);
        assert_eq!(picked.len(), 3);
        let mut uniq = picked.clone();
        uniq.sort_unstable();
        uniq.dedup();
        assert_eq!(uniq.len(), 3, "tidak boleh ada duplikat");
    }

    #[test]
    fn range_respects_bounds() {
        let mut r = Rng::new(99);
        for _ in 0..500 {
            let v = r.range(3, 6);
            assert!((3..=6).contains(&v));
        }
    }
}
