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
    let directory = "corpora/wortschatz";

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

    type Chain = Words<(usize, Words<(usize, Words<usize>)>)>;

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

        for ((word1, word2), word3) in sentence.iter().zip(sentence.iter().skip(1)).zip(sentence.iter().skip(2)) {
            let successors = word_chains.entry(word1.clone())
                .or_insert_with(|| (0, HashMap::new()));

            successors.0 += 1;

            let successors2 = successors.1.entry(word2.clone())
                .or_insert_with(|| (0, HashMap::new()));


            successors2.0 += 1;

            let count = successors2.1.entry(word3.clone())
                .or_insert(0);

            *count += 1;
        }
    }

    println!("analyzed all files");
    println!("processed {} words", word_count);
    println!();


    println!("returning early");
    return;


    println!("you type, i predict.");

    fn map_to_sorted_vec<T>(map: impl IntoIterator<Item=(T, usize)>) -> Vec<T> {
        let tree: BTreeMap<usize, T> = map.into_iter()
            .filter(|(_, count)| *count > 1)
            .map(|(value, count)| (count, value)).collect();

        tree.into_iter().map(|(_, value)| value).rev().collect()
    }

    let starter_words = map_to_sorted_vec(sentence_starter_words.into_iter());

    println!("why not start with one of these words: {}?", &starter_words[..42].join(", "));
    println!("type something!");


    let chains: HashMap<String, Vec<String>> = word_chains.iter()
        .map(|(word, (_count, successors))|{
            let successors = successors.iter()
                .map(|(word, (count, _successors))| (word.to_string(), *count));

            (word.to_string(), map_to_sorted_vec(successors))
        })
        .filter(|(_, values)| !values.is_empty())
        .collect();

    let chains2: HashMap<(String, String), Vec<String>> = word_chains.iter()
        .flat_map(|(word1, (_count, successors1))|{
            successors1.iter()
                .map(move |(word2, (_count, successors2))|{
                    ((word1.to_string(), word2.to_string()), map_to_sorted_vec(successors2.clone()))
                })
        })
        .filter(|(_, values)| !values.is_empty())
        .collect();


    loop {
        let input = {
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        };

        let words = split_to_words(&input);
        let two_words: Vec<String> = words.into_iter().rev().take(2).rev().collect();

        match two_words.as_slice() {
            [word] => {
                let options = chains.get(word).map(|successors|{
                    &successors[ .. successors.len().min(6) ]
                });

                if let Some(options) = options {
                    for option in options {
                        if let Some(option2) = chains2.get(&(word.to_string(), option.to_string())).and_then(|successors| successors.first()) {
                            println!("\t... {} {}", option, option2);
                        }
                        else {
                            println!("\t... {}", option);
                        }
                    }
                }
                else {
                    println!("sorry, i have no idea what you want to type")
                }

                println!()
            },

            [word1, word2] => {
                let options = chains2.get(&(word1.to_string(), word2.to_string())).map(|successors|{
                    &successors[ .. successors.len().min(6) ]
                });

                if let Some(options) = options {
                    for option1 in options {
                        if let Some(option2) = chains2.get(&(word2.to_string(), option1.to_string())).and_then(|successors| successors.first()) {
                            println!("\t... {} {}", option1, option2);
                        }
                        else {
                            println!("\t... {}", option1);
                        }
                    }
                }
                else {
                    println!("sorry, i have no idea what you want to type")
                }

                println!()
            },

            [] => continue,
            _ => unreachable!(),
        };

    }
}
