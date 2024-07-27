use kanji_iter::ToKanji;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    error::Error,
    fs,
};
mod kanji_iter;

fn main() -> Result<(), Box<dyn Error>> {
    let json_data = fs::read_to_string("kanji.json")?;
    let kanji_map: HashMap<String, KanjiEntry> = serde_json::from_str(&json_data)?;
    store_json(json!(kanji_map), "kanji_dict.json");
    // check_anki(&kanji_map);
    if let Some(anki_kanji) = anki_kanji_mapper(&kanji_map, "grade_6.json") {
        let todo_kanji: Vec<KanjiEntry> = kanji_map
            .iter()
            .filter_map(|(_, kanji)| {
                if kanji.grade == 6 {
                    Some(kanji.clone())
                } else {
                    None
                }
            })
            .filter(|kanji| !anki_kanji.contains(&kanji))
            .collect();
        store_json(json!(todo_kanji), "todo_6.json");
        todo_kanji.iter().for_each(|k| println!("{:#?}", k));
        println!("{}", todo_kanji.len());
    }
    Ok(())
}

fn tobira_kanji(
    kanji_map: &HashMap<String, KanjiEntry>,
) -> Result<Vec<KanjiEntrySimple>, Box<dyn Error>> {
    let mut tobira_kanji: Vec<KanjiEntrySimple> = read_csv_kanji("tobira_kanji.csv")?
        .iter()
        .flat_map(|s| s.chars())
        .kanji_map(&kanji_map)
        .iter()
        .map(KanjiEntrySimple::from)
        .collect();
    tobira_kanji.sort();
    Ok(tobira_kanji)
}

fn store_json(value: Value, file_name: &str) {
    fs::write(file_name, value.to_string());
}

fn check_anki(kanji_map: &HashMap<String, KanjiEntry>) {
    let anki_kanji = anki_kanji_mapper(&kanji_map, "grade4kanjideck.json");
    if let Some(kanji) = anki_kanji {
        let correct = kanji.iter().filter(|k| k.grade == 4);
        let incorrect = kanji.iter().filter(|k| k.grade > 4);
        println!("{:#?}", correct.count());
        incorrect.for_each(|k| println!("{:#?}", k));
    }
}

fn anki_kanji_mapper(map: &HashMap<String, KanjiEntry>, file: &str) -> Option<Vec<KanjiEntry>> {
    let anki_grade4_kanji = extract_anki_deck_answers(file);
    if let Some(kanji_strings) = anki_grade4_kanji {
        Some(kanji_strings.iter().flat_map(|s| s.chars()).kanji_map(map))
    } else {
        None
    }
}

fn read_csv_kanji(file: &str) -> Result<HashSet<String>, Box<dyn Error>> {
    let mut rdr = csv::Reader::from_path(file)?;
    let mut kanji_set = HashSet::new();
    for record in rdr.records() {
        kanji_set.insert(record.map(|x| x.iter().collect::<String>())?);
    }
    Ok(kanji_set)
}

fn extract_anki_deck_answers(file_path: &str) -> Option<Vec<String>> {
    let file_str = fs::read_to_string(file_path).ok()?;
    let json: Value = serde_json::from_str(&file_str).ok()?;
    if let Some(notes) = json["notes"].as_array() {
        Some(
            notes
                .iter()
                .filter_map(|note| extract_last_field(note))
                .collect(),
        )
    } else {
        None
    }
}

fn extract_last_field(note: &Value) -> Option<String> {
    note["fields"]
        .as_array()
        .and_then(|fields| fields.last())
        .and_then(|last_field| last_field.as_str())
        .and_then(|last_field| Some(last_field.to_string()))
}

fn is_kanji(c: &char) -> bool {
    ('\u{4e00}'..='\u{9faf}').contains(c)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KanjiEntrySimple {
    pub literal: String,
    pub grade: i32,
    pub stroke_count: i32,
    pub onyomi: String,
    pub kunyomi: String,
    pub english_meanings: String,
}

impl From<&KanjiEntry> for KanjiEntrySimple {
    fn from(entry: &KanjiEntry) -> Self {
        KanjiEntrySimple {
            literal: entry.literal.clone(),
            grade: entry.grade,
            stroke_count: entry.stroke_count,
            onyomi: entry.onyomi.join(", "),
            kunyomi: entry.kunyomi.join(", "),
            english_meanings: entry
                .english_meanings
                .iter()
                .cloned()
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
}

impl Ord for KanjiEntrySimple {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.grade < 1 && other.grade >= 1 {
            return Ordering::Greater; // Place entries with grade 0 or lower at the end.
        } else if self.grade >= 1 && other.grade < 1 {
            return Ordering::Less; // Place entries with grade 0 or lower at the end.
        } else if self.grade != other.grade {
            return self.grade.cmp(&other.grade); // Compare by grade.
        } else {
            return self.stroke_count.cmp(&other.stroke_count); // If grades are equal, compare by stroke count.
        }
    }
}

impl PartialOrd for KanjiEntrySimple {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KanjiEntry {
    pub literal: String,
    pub grade: i32,
    pub stroke_count: i32,
    pub onyomi: Vec<String>,
    pub kunyomi: Vec<String>,
    pub english_meanings: HashSet<String>,
}
