use std::{
    collections::{HashMap, HashSet},
    slice,
};

use crate::{is_kanji, KanjiEntry, KanjiEntrySimple};

pub trait ToKanji<I>
where
    I: Iterator<Item = char>,
{
    fn kanji_map(&mut self, map: &HashMap<String, KanjiEntry>) -> Vec<KanjiEntry>;
}

impl<I> ToKanji<I> for I
where
    I: Iterator<Item = char>,
{
    fn kanji_map(&mut self, map: &HashMap<String, KanjiEntry>) -> Vec<KanjiEntry> {
        self.filter(is_kanji)
            .collect::<HashSet<char>>()
            .iter()
            .filter_map(|c| map.get(&c.to_string()))
            .map(KanjiEntry::clone)
            .collect()
    }
}
