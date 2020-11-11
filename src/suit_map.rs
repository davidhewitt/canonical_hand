use crate::Suit;
use num_traits::FromPrimitive;

// Map from suit to some value
#[derive(Copy, Clone, Debug)]
pub(crate) struct SuitMap<T>([T; 4]);

impl<T: Copy> SuitMap<T> {
    pub(crate) fn new_copied(value: T) -> Self {
        Self([value; 4])
    }
}

impl<T> SuitMap<T> {
    pub(crate) fn get(&self, suit: Suit) -> &T {
        &self.0[suit as usize]
    }

    pub(crate) fn get_mut(&mut self, suit: Suit) -> &mut T {
        &mut self.0[suit as usize]
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (Suit, &T)> {
        self.0
            .iter()
            .enumerate()
            .map(|(idx, value)| (Suit::from_usize(idx).unwrap(), value))
    }

    pub(crate) fn map<U>(self, f: impl FnMut(T) -> U) -> SuitMap<U> {
        self.0.map(f).into()
    }
}

/// Interpret array of 4 values as mapping Clubs -> x[0], Diamonds -> x[1], Hearts -> x[2], Spades -> x[3]
impl<T> From<[T; 4]> for SuitMap<T> {
    fn from(other: [T; 4]) -> Self {
        Self(other)
    }
}
