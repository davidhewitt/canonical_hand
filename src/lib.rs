#![feature(is_sorted)]
#![feature(array_map)]

use num_derive::FromPrimitive;
use std::fmt::Debug;
use strum::{EnumIter, IntoEnumIterator};

use Suit::*;
use Value::*;

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Eq, Ord, Hash, FromPrimitive, EnumIter)]
pub enum Value {
    Two = 2,
    Three = 3,
    Four = 4,
    Five = 5,
    Six = 6,
    Seven = 7,
    Eight = 8,
    Nine = 9,
    Ten = 10,
    Jack = 11,
    Queen = 12,
    King = 13,
    Ace = 14,
}

impl Value {
    pub const fn of(self, suit: Suit) -> Card {
        Card { value: self, suit }
    }

    const fn shorthand(self) -> &'static str {
        match self {
            Two => "2",
            Three => "3",
            Four => "4",
            Five => "5",
            Six => "6",
            Seven => "7",
            Eight => "8",
            Nine => "9",
            Ten => "10",
            Jack => "J",
            Queen => "Q",
            King => "K",
            Ace => "A",
        }
    }
}

#[derive(Debug, PartialOrd, PartialEq, Copy, Clone, Eq, Ord, Hash, FromPrimitive, EnumIter)]
pub enum Suit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

impl Suit {
    const fn shorthand(self) -> &'static str {
        match self {
            Clubs => "C",
            Diamonds => "D",
            Hearts => "H",
            Spades => "S",
        }
    }
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Eq, Ord, Hash)]
pub struct Card {
    value: Value,
    suit: Suit
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Card({}{})", self.value.shorthand(), self.suit.shorthand())
    }
}

pub const CANONICAL_DECK: [Card; 52] = [
    Two.of(Clubs),
    Two.of(Diamonds),
    Two.of(Hearts),
    Two.of(Spades),
    Three.of(Clubs),
    Three.of(Diamonds),
    Three.of(Hearts),
    Three.of(Spades),
    Four.of(Clubs),
    Four.of(Diamonds),
    Four.of(Hearts),
    Four.of(Spades),
    Five.of(Clubs),
    Five.of(Diamonds),
    Five.of(Hearts),
    Five.of(Spades),
    Six.of(Clubs),
    Six.of(Diamonds),
    Six.of(Hearts),
    Six.of(Spades),
    Seven.of(Clubs),
    Seven.of(Diamonds),
    Seven.of(Hearts),
    Seven.of(Spades),
    Eight.of(Clubs),
    Eight.of(Diamonds),
    Eight.of(Hearts),
    Eight.of(Spades),
    Nine.of(Clubs),
    Nine.of(Diamonds),
    Nine.of(Hearts),
    Nine.of(Spades),
    Ten.of(Clubs),
    Ten.of(Diamonds),
    Ten.of(Hearts),
    Ten.of(Spades),
    Jack.of(Clubs),
    Jack.of(Diamonds),
    Jack.of(Hearts),
    Jack.of(Spades),
    Queen.of(Clubs),
    Queen.of(Diamonds),
    Queen.of(Hearts),
    Queen.of(Spades),
    King.of(Clubs),
    King.of(Diamonds),
    King.of(Hearts),
    King.of(Spades),
    Ace.of(Clubs),
    Ace.of(Diamonds),
    Ace.of(Hearts),
    Ace.of(Spades),
];

#[derive(Copy, Clone, Debug)]
struct SuitPermutation([Suit; 4]);

impl SuitPermutation {
    /// Returns a new SuitPermutation from the unders
    ///
    /// target_suits is interpreted as the suit the corresponding original
    /// suit should be output as, where the "starting order" is
    ///      [Clubs, Diamonds, Hearts, Spades]
    ///
    /// e.g. [Hearts, Diamonds, Spades, Clubs] implies
    ///   Clubs => Hearts
    ///   Diamonds => Diamonds
    ///   Hearts => Spades,
    ///   Spades => Clubs
    fn new(target_suits: [Suit; 4]) -> Self {
        let mut seen_targets = [false; 4];
        for target in &target_suits {
            seen_targets[*target as usize] = true;
        }
        assert!(
            seen_targets.iter().all(|seen| *seen),
            "target_suits must contain all four suits"
        );

        Self(target_suits)
    }

    pub fn map(&self, suit: Suit) -> Suit {
        self.0[suit as usize]
    }
}

/// Permute cards to a new suit variation
fn permute_suits(mut cards: Vec<Card>, permutation: SuitPermutation) -> Vec<Card> {
    for card in &mut cards {
        card.suit = permutation.map(card.suit);
    }

    cards
}

/// Get strategically equivalent hand with lexicographic minimum
pub fn canonicalize_hand(mut cards: Vec<Card>) -> Vec<Card> {
    // map from original suit (by index) to assigned suit
    let mut assigned_suits: [Option<Suit>; 4] = [None; 4];

    // next suit generator
    let mut suit_generator = Suit::iter();

    // sort hand cards
    cards[0..2].sort();

    // sort table cards
    cards[2..].sort();

    dbg!(&cards);

    let mut rest = &cards[..];

    'per_card: loop {
        rest = match rest.split_first() {
            Some((card, remaining)) => {
                while assigned_suits[card.suit as usize].is_none() {
                    match get_next_suit_to_assign(card, remaining, &assigned_suits) {
                        Some(suit) => assigned_suits[suit as usize] = suit_generator.next(),
                        // All remaining cards are ambiguous, just let the assigned suits fill by the permutation
                        None => break 'per_card
                    }
                }
                remaining
            }
            None => break,
        }
    }

    let permutation = SuitPermutation::new(
        assigned_suits.map(|suit| suit.or_else(|| suit_generator.next()).unwrap())
    );

    dbg!(&permutation);

    cards = permute_suits(cards, permutation);

    // sort cards again - pairs mean the original sort is not guaranteed to be correct any more
    cards[0..2].sort();
    cards[2..].sort();

    cards
}

#[inline]
fn get_next_suit_to_assign(
    card: &Card,
    remaining: &[Card],
    assigned_suits: &[Option<Suit>; 4],
) -> Option<Suit> {
    // This card's suit is already assigned, look at next card
    if assigned_suits[card.suit as usize].is_some() {
        return remaining.split_first().and_then(|(card, remaining)| {
            get_next_suit_to_assign(card, remaining, assigned_suits)
        });
    }

    // This card's suit is not assigned; search against future cards for pairs, triples etc.

    match remaining.split_first() {
        Some((mut future_card, mut remaining)) => {
            let is_ambiguous_group = false;

            // TODO
            Some(card.suit)
        },
        // No future cards, so return this card's suit
        None => Some(card.suit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;
    use proptest::prelude::*;
    use std::convert::TryInto;
    use std::collections::HashMap;

    #[test]
    fn canonical_deck_sorted() {
        assert!(CANONICAL_DECK.is_sorted());
    }

    #[test]
    fn canonical_deck_no_duplicates() {
        // As long as canonical_deck is sorted, this will remove all duplicates.
        let deduped = {
            let mut owned = CANONICAL_DECK.to_vec();
            owned.dedup();
            owned
        };
        assert!(deduped.len() == CANONICAL_DECK.len());
    }

    // proptesting strategies

    fn any_card() -> impl Strategy<Value = Card> {
        (
            Two as usize..=Ace as usize,
            Diamonds as usize..=Spades as usize,
        )
            .prop_map(|(value, suit)| {
                Card {
                    value: Value::from_usize(value).unwrap(),
                    suit: Suit::from_usize(suit).unwrap(),
                }
            })
    }

    fn any_suit_permutation() -> impl Strategy<Value = SuitPermutation> {
        Just([Clubs, Diamonds, Hearts, Spades].to_vec())
            .prop_shuffle()
            .prop_map(|perm_vec| SuitPermutation::new(perm_vec.as_slice().try_into().unwrap()))
    }

    // proptests for permute_suits

    proptest! {
        #[test]
        fn test_permute_suits_counts(
            cards in prop::collection::vec(any_card(), 0..=1000),
            permutation in any_suit_permutation(),
        ) {
            // permutation should always preserve count of values
            // permutation will map count of suits
            let mut original_suit_counts = HashMap::new();
            let mut original_value_counts = HashMap::new();
            for card in &cards {
                *original_suit_counts.entry(card.suit).or_insert(0) += 1;
                *original_value_counts.entry(card.value).or_insert(0) += 1;
            }

            let permuted_cards = permute_suits(cards, permutation);

            let mut permuted_suit_counts = HashMap::new();
            let mut permuted_value_counts = HashMap::new();
            for card in &permuted_cards {
                *permuted_suit_counts.entry(card.suit).or_insert(0) += 1;
                *permuted_value_counts.entry(card.value).or_insert(0) += 1;
            }

            assert_eq!(original_value_counts, permuted_value_counts);
            for (suit, count) in original_suit_counts {
                assert_eq!(count, permuted_suit_counts[&permutation.map(suit)]);
            }
        }

        #[test]
        fn test_permute_suits_cyclic(
            mut cards in prop::collection::vec(any_card(), 0..=1000),
            permutation in any_suit_permutation(),
        ) {
            // A permutation with four suits will always be cyclic with period of
            // at most 4.
            let original_cards = cards.clone();
            let mut good = false;

            for _ in 0..4 {
                cards = permute_suits(cards, permutation);
                if cards == original_cards {
                    good = true;
                    break;
                }
            }

            if !good {
                panic!("cards were not reordered after 4 permutations");
            }
        }
    }

    // proptests for canonicalize_hand

    prop_compose! {
        fn any_hand()(
            shuffled_deck in Just(CANONICAL_DECK.to_vec()).prop_shuffle(),
            dealt_cards in prop::sample::select(&[2, 5, 6, 7][..]),
        ) -> Vec<Card> {
            shuffled_deck[0..dealt_cards].to_vec()
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(10000))]

        #[test]
        fn test_canonicalize_hand_lexicographic_minimum_general(
            hand in any_hand(),
            permutation in any_suit_permutation(),
        ) {
            dbg!(&hand);
            let canonical = canonicalize_hand(hand);
            let mut permuted = permute_suits(canonical.clone(), permutation);
            permuted[0..2].sort();
            permuted[2..].sort();
            dbg!((&canonical, &permuted));
            assert!(canonical <= permuted);
        }
    }
    #[test]
    fn test_canonicalize_hand_perverse_case_one() {
        let hand = vec![
            Two.of(Clubs),
            Two.of(Spades),
            Three.of(Clubs),
            Three.of(Spades),
            Three.of(Diamonds),
        ];
        let canonical = canonicalize_hand(hand.clone());

        assert_eq!(
            canonical,
            vec![
                Two.of(Clubs),
                Two.of(Diamonds),
                Three.of(Clubs),
                Three.of(Diamonds),
                Three.of(Hearts)
            ]
        );
    }

    // #[test]
    // fn test_canonicalize_hand_perverse_case_one() {
    //     let hand = vec![
    //         Two.of(Clubs),
    //         Two.of(Spades),
    //         Three.of(Clubs),
    //         Three.of(Spades),
    //         Three.of(Diamonds),
    //     ];
    //     let canonical = canonicalize_hand(hand.clone());

    //     assert_eq!(
    //         canonical,
    //         vec![
    //             Two.of(Clubs),
    //             Two.of(Diamonds),
    //             Three.of(Clubs),
    //             Three.of(Diamonds),
    //             Three.of(Hearts)
    //         ]
    //     );
    // }
}
