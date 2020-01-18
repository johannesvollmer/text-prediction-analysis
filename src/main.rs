// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/

use std::collections::{HashMap, BTreeMap};
use std::ffi::OsStr;
use std::path::PathBuf;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::io;
use std::io::{BufReader, BufRead};
use std::fs::File;

fn main() {
    let directory = "corpora/oanc";

    let files: Vec<(usize, PathBuf)> = walkdir::WalkDir::new(directory).into_iter().map(Result::unwrap)
        .filter(|entry| entry.path().extension() == Some(OsStr::new("txt")))
        .map(walkdir::DirEntry::into_path).enumerate()
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


    files.into_par_iter().for_each_with(sender, |sentence_sender, (_index, path)| {
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


    type Words<T> = HashMap<String, T>;
    type WordCount = Words<usize>;
    type CharCount = HashMap<char, usize>;
    type Chain = HashMap<Vec<String>, WordCount>;
    let max_chain_len = 2;

    let mut word_chains: Chain = Chain::with_capacity(1024*1024);
    let mut sentence_starter_words: WordCount = HashMap::with_capacity(1024*1024);
    let mut sentence_starter_chars: CharCount = HashMap::new();
    let mut word_starter_chars: CharCount = HashMap::new();

    let mut word_count: u128 = 0;

    for sentence in sentence_receiver {
        word_count += sentence.len() as u128;

        *sentence_starter_words.entry(sentence.first().unwrap().clone()).or_insert(0) += 1;
        *sentence_starter_chars.entry(sentence.first().unwrap().chars().next().unwrap()).or_insert(0) += 1;

        for word in &sentence {
            *word_starter_chars.entry(word.chars().next().unwrap()).or_insert(0) += 1;
        }

        for chain_len in 1 ..= max_chain_len {
            for key in sentence.windows(chain_len + 1) {
                let value = &key[chain_len];
                let key = Vec::from(&key[ .. chain_len]);

                let map = word_chains.entry(key.clone()).or_insert_with(HashMap::new);
                *map.entry(value.clone()).or_insert(0) += 1;
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

    println!("why not start with one of these words: {}?", &starter_words[..42].join(", "));
    println!("type something!");


    let chains: HashMap<Vec<String>, Vec<String>> = word_chains.into_par_iter()
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

        for chain_len in (1 ..= max_chain_len.min(words.len())).rev() {
            let mut key = Vec::from(&words[words.len() - chain_len .. ]);

            let options = chains.get(&key).map(|options|{
                &options[ .. options.len().min(6) ]
            });

            if let Some(options) = options {
                for option in options {
                    key.push(option.clone());

                    if let Some(option2) = chains.get(&key).and_then(|successors| successors.first()) {
                        println!("\t... {} {} (2, exact)", option, option2);
                    }
                    else {
                        key.remove(0);

                        if let Some(option2) = chains.get(&key).and_then(|successors| successors.first()) {
                            println!("\t... {} {} (2, artificial)", option, option2);
                        }
                        else {
                            println!("\t... {} (1)", option);
                        }
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
