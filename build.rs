use std::{
    collections::{HashMap, HashSet},
    fs,
};

use roxmltree::{Document, ParsingOptions};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Clone, Serialize)]
pub struct KanjiEntry {
    pub literal: String,
    pub grade: i32,
    pub stroke_count: i32,
    pub onyomi: Vec<String>,
    pub kunyomi: Vec<String>,
    pub english_meanings: HashSet<String>,
}

fn parse_kanji_xml() -> HashMap<String, KanjiEntry> {
    let xml = fs::read_to_string("kanjidic2.xml").unwrap();
    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let doc = Document::parse_with_options(&xml, options).unwrap();
    let mut kanji_map = HashMap::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("character")) {
        let literal = node
            .descendants()
            .find(|n| n.has_tag_name("literal"))
            .map(|n| n.text().unwrap_or_default().to_string())
            .unwrap_or_default();

        let grade = node
            .descendants()
            .find(|n| n.has_tag_name("grade"))
            .map(|n| n.text().unwrap_or_default().parse().unwrap_or(0))
            .unwrap_or(0);

        let stroke_count = node
            .descendants()
            .find(|n| n.has_tag_name("stroke_count"))
            .map(|n| n.text().unwrap_or_default().parse().unwrap_or(0))
            .unwrap_or(0);

        let mut onyomi = Vec::new();
        let mut kunyomi = Vec::new();
        let mut english_meanings = HashSet::new();

        for rmgroup_node in node.descendants().filter(|n| n.has_tag_name("rmgroup")) {
            for reading_node in rmgroup_node
                .descendants()
                .filter(|n| n.has_tag_name("reading"))
            {
                let value = reading_node.text().unwrap_or_default();
                if let Some(r_type) = reading_node.attribute("r_type") {
                    match r_type {
                        "ja_on" => onyomi.push(value.to_string()),
                        "ja_kun" => kunyomi.push(value.to_string()),
                        _ => {}
                    }
                }
            }
            for meaning_node in rmgroup_node
                .descendants()
                .filter(|n| n.has_tag_name("meaning"))
            {
                // Check if there's no additional language specification in the meaning tag
                if !meaning_node.has_attribute("m_lang") {
                    let meaning = meaning_node.text().unwrap_or_default().to_string();
                    english_meanings.insert(meaning);
                }
            }
        }

        kanji_map.insert(
            literal.clone(),
            KanjiEntry {
                literal,
                grade,
                stroke_count,
                onyomi,
                kunyomi,
                english_meanings,
            },
        );
    }

    kanji_map
}

fn main() {
    let kanji_map = parse_kanji_xml();
    let reading_json = json!(kanji_map);
    fs::write("kanji.json", reading_json.to_string()).unwrap();
    // println!("cargo:rerun-if-changed=kanjidic2.xml");
}
