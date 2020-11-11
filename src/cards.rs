use num_derive::FromPrimitive;
use strum::EnumIter;

use std::fmt::Debug;

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
    pub value: Value,
    pub suit: Suit,
}

impl Debug for Card {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Card({}{})",
            self.value.shorthand(),
            self.suit.shorthand()
        )
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
