// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt


use std::collections::{HashMap, BTreeMap};
use std::ffi::OsStr;
use std::{io, mem};
use std::io::{BufReader, BufRead};
use std::fs::File;
use string_interner::StringInterner;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

mod correction;

fn main() {
    let directory = "corpora";
    println!("beginning text analysis of directory `{}`", directory);


    let files = walkdir::WalkDir::new(directory).into_iter().map(Result::unwrap)
        .filter(|entry| entry.path().extension() == Some(OsStr::new("txt")))
        .map(walkdir::DirEntry::into_path);

    let sentences = files.into_iter().flat_map(|path| {
        let mut chars = BufReader::new(File::open(path).unwrap())
            .lines().flat_map(|string| string.unwrap().chars().collect::<Vec<char>>().into_iter());

        std::iter::from_fn(move || {
            let mut sentence = String::with_capacity(256);

            while let Some(character) = chars.next() {
                if "!?.".contains(character) {
                    return Some(split_to_words(
                        &sentence.replace("-\n", "") // merge words that have been split by a linebreak
                    ));
                }
                else {
                    sentence.push(character);
                }
            }

            return None;
        })
    });

    let sentences = sentences.filter_map(|sentence|{
        let sentence = sentence.into_iter().filter(|word| !word.is_empty()).collect::<Vec<String>>();
        if !sentence.is_empty() { Some(sentence) }
        else { None }
    });

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



    type StringId = usize;
    type Count<T> = HashMap<T, usize>;
    type Chain<T> = HashMap<Vec<T>, Count<T>>;
    let max_chain_len = 2;

    let mut strings: StringInterner<StringId> = string_interner::StringInterner::with_capacity(2048);

    // initialize the statistical data which we are going to analyze
    let mut word_chains: Chain<StringId> = Chain::with_capacity(1024*1024);
    let mut sentence_starter_words: Count<StringId> = HashMap::with_capacity(1024*1024);
    let mut sentence_starter_chars: Count<char> = HashMap::new();
    let mut word_starter_chars: Count<char> = HashMap::new();
    let mut word_count: u128 = 0;

    // synchroneously collect all the parsed data into our statistical hashmap
    for sentence in sentences {
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
    println!("collected {} prediction entries", word_chains.len());
    println!();


    println!("you type, i predict.");

    fn map_to_sorted_vec<T>(map: Count<T>) -> Vec<T> {
        let tree: BTreeMap<usize, T> = map.into_iter()
            .map(|(value, count)| (count, value)).collect();

        tree.into_iter().map(|(_, value)| value).rev().collect()
    }

    let starter_words = map_to_sorted_vec(sentence_starter_words);
    let starter_word_strings: Vec<&str> = starter_words.iter().map(|&id| strings.resolve(id).unwrap()).collect();

    println!("why not start with one of these words: {}?", &starter_word_strings[..24].join(", "));
    println!("type something!");


    let chains: HashMap<Vec<StringId>, Vec<StringId>> = word_chains
        .into_par_iter().filter(|(_, values)| !values.is_empty())
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
