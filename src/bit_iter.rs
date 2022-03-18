const BIT_MASK: [u8; 8] = [1, 2, 4, 8, 16, 32, 64, 128];

pub trait BitIter {
    /// Iterate over the bits of this integer, from high to low.
    fn iter_bits(self) -> IterBits;
}

impl BitIter for u8 {
    fn iter_bits(self) -> IterBits {
        IterBits { value: self, i: 0 }
    }
}

pub struct IterBits {
    value: u8,
    i: u8,
}

impl Iterator for IterBits {
    type Item = bool;

    fn next(&mut self) -> Option<Self::Item> {
        if self.i < 8 {
            let is_set = self.value & BIT_MASK[self.i as usize] != 0;
            self.i += 1;

            Some(is_set)
        } else {
            None
        }
    }
}

#[test]
fn test_iter_bits() {
    let bits: Vec<bool> = 7.iter_bits().collect();
    assert_eq!(bits, &[true, true, true, false, false, false, false, false]);
    let bits: Vec<bool> = 9.iter_bits().collect();
    assert_eq!(
        bits,
        &[true, false, false, true, false, false, false, false]
    );
}
