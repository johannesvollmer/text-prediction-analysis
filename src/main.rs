// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/

use std::collections::{HashMap, BTreeMap};
use std::ffi::OsStr;
use std::path::{PathBuf};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::io;
use std::io::{BufReader, BufRead};
use std::fs::File;
use string_interner::StringInterner;

fn main() {
    let directory = "corpora";
    let files: Vec<PathBuf> = walkdir::WalkDir::new(directory).into_iter().map(Result::unwrap)
        .filter(|entry| entry.path().extension() == Some(OsStr::new("txt")))
        .map(walkdir::DirEntry::into_path)
        .collect();

    let count = files.len();
    println!("starting to analyze {} files", count);

    let (sender, sentence_receiver) = std::sync::mpsc::channel();

    fn split_to_words(sentence: &str) -> Vec<String> {
        sentence.split(char::is_whitespace)
            .map(|word|
                word.chars()
                    .flat_map(|c| c.to_lowercase())
                    .filter(|&c| c.is_alphabetic() || c == '\'')
                    .collect::<String>()
            )
            .filter(|w| w.len() > 0 && w != "\'")
            .collect()
    }


    files.into_par_iter().for_each_with(sender, |sentence_sender, path| {
        let mut chars = BufReader::new(File::open(path).unwrap())
            .lines().flat_map(|string| string.unwrap().chars().collect::<Vec<char>>().into_iter());

        let mut sentence = String::with_capacity(1024);

        while let Some(character) = chars.next() {
            if "!?.".contains(character) {
                sentence = sentence.replace("-\n", ""); // merge words that have been split by a linebreak
                sentence = sentence.trim().to_string();

                if !sentence.is_empty() {
                    let words = split_to_words(&sentence);

                    if !words.is_empty() {
                        sentence_sender.send(words).unwrap();
                    }

                    sentence.clear();
                }
            }
            else {
                sentence.push(character);
            }
        }
    });



    type StringId = usize;
    type Words<T> = HashMap<StringId, T>;
    type WordCount = Words<usize>;
    type CharCount = HashMap<char, usize>;
    type Chain = HashMap<Vec<StringId>, WordCount>;
    let max_chain_len = 1;


    let mut strings: StringInterner<StringId> = string_interner::StringInterner::with_capacity(2048);

    let mut word_chains: Chain = Chain::with_capacity(1024*1024);
    let mut sentence_starter_words: WordCount = HashMap::with_capacity(1024*1024);
    let mut sentence_starter_chars: CharCount = HashMap::new();
    let mut word_starter_chars: CharCount = HashMap::new();

    let mut word_count: u128 = 0;

    for sentence in sentence_receiver {
        let words: Vec<StringId> = sentence.iter().map(|string| strings.get_or_intern(string)).collect();

        word_count += sentence.len() as u128;

        *sentence_starter_words.entry(*words.first().unwrap()).or_insert(0) += 1;
        *sentence_starter_chars.entry(sentence.first().unwrap().chars().next().unwrap()).or_insert(0) += 1;

        for word in &sentence {
            *word_starter_chars.entry(word.chars().next().unwrap()).or_insert(0) += 1;
        }

        for chain_len in 1 ..= max_chain_len {
            for key in words.windows(chain_len + 1) {
                let value = &key[chain_len];
                let key = Vec::from(&key[ .. chain_len]);

                let map = word_chains.entry(key).or_insert_with(HashMap::new);
                *map.entry(*value).or_insert(0) += 1;
            }
        }
    }

    println!("analyzed all files");
    println!("processed {} words", word_count);
    println!();


    println!("you type, i predict.");

    fn map_to_sorted_vec<T>(map: HashMap<T, usize>) -> Vec<T> {
        let tree: BTreeMap<usize, T> = map.into_iter()
            .map(|(value, count)| (count, value)).collect();

        tree.into_iter().map(|(_, value)| value).rev().collect()
    }

    let starter_words = map_to_sorted_vec(sentence_starter_words);
    let starter_word_strings: Vec<&str> = starter_words.iter().map(|&id| strings.resolve(id).unwrap()).collect();

    println!("why not start with one of these words: {}?", &starter_word_strings[..24].join(", "));
    println!("type something!");


    let chains: HashMap<Vec<StringId>, Vec<StringId>> = word_chains.into_par_iter()
        .filter(|(_, values)| !values.is_empty())
        .map(|(words, successors)| (words, map_to_sorted_vec(successors)))
        .collect();

    loop {
        let input = {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        };

        let words = split_to_words(&input);
        let word_ids: Vec<StringId> = words.iter().map(|string| strings.get_or_intern(string)).collect();

        for chain_len in (1 ..= max_chain_len.min(word_ids.len())).rev() {
            let mut key: Vec<StringId> = Vec::from(&word_ids[word_ids.len() - chain_len .. ]);

            let options = chains.get(&key).map(|options|{
                &options[ .. options.len().min(20) ]
            });

            if let Some(options) = options {
                for &option in options {
                    key.push(option.clone());

                    if let Some(&option2) = chains.get(&key).and_then(|successors| successors.first()) {
                        println!("\t... {} {}", strings.resolve(option).unwrap(), strings.resolve(option2).unwrap());
                    }
                    else {
//                        key.remove(0);
//                        if let Some(&option2) = chains.get(&key).and_then(|successors| successors.first()) {
//                            println!("\t... {} {} (2, artificial)", strings.resolve(option).unwrap(), strings.resolve(option2).unwrap());
//                        }
//                        else {
                            println!("\t... {}", strings.resolve(option).unwrap());
//                        }
                    }
                }
            }
            else {
                println!("sorry, i have no idea what you want to type");
            }

            break;
        };

    }
}
