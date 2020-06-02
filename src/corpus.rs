
use std::ffi::OsStr;
use std::io::{BufReader, BufRead};
use std::fs::File;

pub fn sentences() -> impl Iterator<Item = String> {
    let directory = "corpora/norvig-com-big.txt"; // TODO
    // let directory = "corpora";

    let files = walkdir::WalkDir::new(directory)
        .into_iter().filter_entry(|entry| !entry.path().file_name().unwrap().to_str().unwrap().starts_with("_"))
        .map(Result::unwrap).filter(|entry| entry.path().extension() == Some(OsStr::new("txt"))) // ignore xml files
        .map(walkdir::DirEntry::into_path);

    let sentences = files.into_iter().flat_map(|path| {
        let mut chars = BufReader::new(File::open(path).unwrap())
            .lines().flat_map(|string| string.unwrap().chars().collect::<Vec<char>>().into_iter());

        std::iter::from_fn(move || {
            let mut sentence = String::with_capacity(256);

            while let Some(character) = chars.next() {
                if "!?.".contains(character) {
                    let sentence = sentence.replace("-\n", ""); // merge words that have been split by a linebreak
                    return Some(sentence);
                }
                else {
                    sentence.push(character);
                }
            }

            return None;
        })
    });

    sentences.filter_map(|sentence| if !sentence.is_empty() { Some(sentence) } else { None })
}

/// May return an empty string
pub fn split_to_words(sentence: &str) -> Vec<String> {
    sentence.split_whitespace()
        .map(|word|
            word.chars()
                .flat_map(|c| c.to_lowercase())
                .filter(|&c| c.is_alphabetic() || c == '\'')
                .collect::<String>()
        )
        .filter(|w| !w.is_empty() && w != "\'")
        .collect()
}

pub fn words() -> impl Iterator<Item = String> {
    self::sentences().flat_map(|string| split_to_words(&string))
}