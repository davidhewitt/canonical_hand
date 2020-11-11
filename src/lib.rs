#![feature(is_sorted)]
#![feature(option_result_contains)]
#![feature(bool_to_option)]
#![feature(array_map)]

use std::convert::TryInto;
use strum::IntoEnumIterator;

mod cards;
mod suit_map;

pub use cards::*;
use suit_map::*;

/// Permute cards to a new suit variation
///
/// Returns a new SuitPermutation from the underlying targets.
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
fn permute_suits(mut cards: Vec<Card>, target_suits: SuitMap<Suit>) -> Vec<Card> {
    let mut seen_targets = [false; 4];
    for (_, target) in target_suits.iter() {
        seen_targets[*target as usize] = true;
    }
    assert!(
        seen_targets.iter().all(|seen| *seen),
        "target_suits must contain all four suits"
    );

    for card in &mut cards {
        card.suit = *target_suits.get(card.suit);
    }

    cards
}

/// Get strategically equivalent hand with lexicographic minimum
pub fn canonicalize_hand(mut cards: Vec<Card>) -> Vec<Card> {
    // map from original suit (by index) to assigned suit
    let mut assigned_suits = SuitMap::new_copied(None);

    // sort hand cards
    sort_hand(&mut cards);

    // hole is special case: it can either be resolved immediately, or if a
    // double we need to look ahead to determine correct order
    let hole = &mut cards[0..2].try_into().unwrap();
    if let Some(suit) = hole_cards_same_value(hole).and_then(|ambiguous_group: SuitMap<bool>| {
        find_first_intersection(&cards[2..], ambiguous_group)
    }) {
        // Swap the suits in the double if the second card has the first suit intersecting
        // with the cards on the table.
        if suit == hole[1].suit {
            hole[1].suit = std::mem::replace(&mut hole[0].suit, suit);
        }
    }

    // next suit generator
    let mut suit_generator = {
        let mut iter = Suit::iter();
        move || iter.next().unwrap()
    };

    // Assign suits to hole cards - condition above guarantees that this is correctly ordered
    *assigned_suits.get_mut(hole[0].suit) = Some(suit_generator());
    assigned_suits.get_mut(hole[1].suit).get_or_insert_with(|| suit_generator());

    let mut remaining = &cards[2..];
    while let Some((card, next_remaining)) = remaining.split_first() {
        while assigned_suits.get(card.suit).is_none() {
            let suit = get_next_suit_to_assign(card, next_remaining, &assigned_suits);
            let assigned = assigned_suits.get_mut(suit);
            assert!(assigned.is_none());
            *assigned = Some(suit_generator());
        }

        remaining = next_remaining;
    }

    let permutation =
        assigned_suits.map(|suit| suit.unwrap_or_else(|| suit_generator()));

    cards = permute_suits(cards, permutation);

    // sort cards again - groups mean the original sort is not guaranteed to be correct any more
    sort_hand(&mut cards);

    cards
}

#[inline]
fn get_next_suit_to_assign(
    card: &Card,
    mut remaining: &[Card],
    assigned_suits: &SuitMap<Option<Suit>>,
) -> Suit {
    assert!(assigned_suits.get(card.suit).is_none());

    let mut is_ambiguous_group = false;
    let mut ambiguous_group = SuitMap::new_copied(false);
    *ambiguous_group.get_mut(card.suit) = true;

    while let Some((next_card, next_remaining)) = remaining.split_first() {
        if next_card.value != card.value {
            break;
        }

        if assigned_suits.get(next_card.suit).is_none() {
            is_ambiguous_group = true;
            *ambiguous_group.get_mut(next_card.suit) = true;
        }

        remaining = next_remaining;
    }

    if is_ambiguous_group {
        find_first_intersection(remaining, ambiguous_group).unwrap_or(card.suit)
    } else {
        card.suit
    }
}

#[inline]
fn hole_cards_same_value(hole: &[Card; 2]) -> Option<SuitMap<bool>> {
    (hole[0].value == hole[1].value).then(|| {
        let mut map = SuitMap::new_copied(false);
        *map.get_mut(hole[0].suit) = true;
        *map.get_mut(hole[1].suit) = true;
        map
    })
}

/// Find first suit which intersects "singly" with a group of suits.
///
/// If remaining has a run of cards in the same value, and several of those intersect
/// with `suits`, then that run is used as a new subset to continue searching for the
/// next interection.
///
/// If remaining ends with a run of cards in the same value, and several of those intersect
/// with `suits`, then the lowest suit (by ordering) in the intersection is returned.
///
/// Remaining is expected to be sorted by value.
fn find_first_intersection(remaining: &[Card], mut suits: SuitMap<bool>) -> Option<Suit> {
    let mut group = SuitMap::new_copied(false);
    let mut group_value = None;

    for card in remaining {
        if group_value.is_some() && !group_value.contains(&card.value) {
            // The intersecting group has ended
            if group.iter().filter(|(_, is_present)| **is_present).count() > 1 {
                // But it's still ambiguous, reset to this subset and continue
                suits = group;
                group = SuitMap::new_copied(false);
                group_value = None;
            } else {
                // It resolves the ambiguity !
                break;
            }
        }

        if *suits.get(card.suit) {
            group_value = Some(card.value);
            *group.get_mut(card.suit) = true;
        }
    }

    let found_suit = group
        .iter()
        .find_map(|(suit, is_present)| is_present.then_some(suit));
    found_suit
}

#[inline]
fn sort_hand(hand: &mut [Card]) {
    // sort hole cards
    hand[0..2].sort();
    // sort table
    hand[2..].sort();
}

#[cfg(test)]
mod tests {
    use super::*;
    use num_traits::FromPrimitive;
    use proptest::prelude::*;
    use std::collections::HashMap;

    use Suit::*;
    use Value::*;

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
            .prop_map(|(value, suit)| Card {
                value: Value::from_usize(value).unwrap(),
                suit: Suit::from_usize(suit).unwrap(),
            })
    }

    fn any_suit_permutation() -> impl Strategy<Value = SuitMap<Suit>> {
        Just([Clubs, Diamonds, Hearts, Spades])
            .prop_shuffle()
            .prop_map(Into::into)
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
                assert_eq!(count, permuted_suit_counts[permutation.get(suit)]);
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
        #![proptest_config(ProptestConfig::with_cases(100000))]

        #[test]
        fn test_canonicalize_hand_lexicographic_minimum(
            hand in any_hand(),
            permutation in any_suit_permutation(),
        ) {
            dbg!(&hand);
            let canonical = canonicalize_hand(hand);
            let mut permuted = permute_suits(canonical.clone(), permutation);
            sort_hand(&mut permuted);
            dbg!((&canonical, &permuted));
            assert!(canonical <= permuted);
        }

        #[test]
        fn test_canonicalize_hand_idempotent(
            hand in any_hand(),
        ) {
            // canonicalizing the canonical hand should be an identity operation
            let len = hand.len();
            let canonical = canonicalize_hand(hand);
            let canonical2 = canonicalize_hand(canonical.clone());
            assert_eq!(canonical.len(), len);
            assert_eq!(canonical, canonical2);
        }
    }

    #[test]
    fn test_canonicalize_hand_perverse_case_one() {
        // Ensure that the hole ambiguity can be resolved by the first card of the table.
        let hand = vec![
            Two.of(Clubs),
            Two.of(Spades),
            Three.of(Spades),
            Four.of(Spades),
            Five.of(Spades),
        ];
        let canonical = canonicalize_hand(hand.clone());

        assert_eq!(
            canonical,
            vec![
                Two.of(Clubs),
                Two.of(Diamonds),
                Three.of(Clubs),
                Four.of(Clubs),
                Five.of(Clubs),
            ]
        );
    }

    #[test]
    fn test_canonicalize_hand_perverse_case_two() {
        // Ensure that the hole ambiguity is resolved when the table
        // has an intersecting pair only later in the ordering.
        let hand = vec![
            Two.of(Spades),
            Two.of(Clubs),
            Two.of(Hearts),
            Three.of(Spades),
            Three.of(Diamonds),
        ];
        let canonical = canonicalize_hand(hand.clone());

        assert_eq!(
            canonical,
            vec![
                Two.of(Clubs),
                Two.of(Diamonds),
                Two.of(Hearts),
                Three.of(Clubs),
                Three.of(Spades),
            ]
        );
    }

    #[test]
    fn test_canonicalize_hand_perverse_case_three() {
        // Ensure that the hole ambiguity is resolved when the table
        // has multiple ambiguous groups before an intersection.
        let hand = vec![
            Two.of(Spades),
            Two.of(Clubs),
            Three.of(Spades),
            Three.of(Clubs),
            Four.of(Spades),
            Four.of(Clubs),
            Five.of(Spades),
        ];
        let canonical = canonicalize_hand(hand.clone());

        assert_eq!(
            canonical,
            vec![
                Two.of(Clubs),
                Two.of(Diamonds),
                Three.of(Clubs),
                Three.of(Diamonds),
                Four.of(Clubs),
                Four.of(Diamonds),
                Five.of(Clubs)
            ]
        );
    }
}
