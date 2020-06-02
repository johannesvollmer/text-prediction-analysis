// // TEXT CORPORA SOURCES (17.01.2020):
// // http://www.anc.org/data/oanc/download/
// // http://www.anc.org/data/masc/downloads/data-download/
// // https://wortschatz.uni-leipzig.de/en/download/
// // http://norvig.com/spell-correct.html -> http://norvig.com/big.txt
//
//


use crate::corpus::split_to_words;
use std::collections::{HashMap, BTreeMap};
use rayon::prelude::IntoParallelIterator;
use std::path::Path;
use std::fs::File;
use crate::corpus;

const MAX_CHAIN_LEN: usize = 2;

pub fn ngram_predictor() -> impl (Fn(&[String]) -> Vec<String>) {
    let path = Path::new(".prediction-cache");
    type StringId = usize;

    let (starters, strings, chains, top_words) = {
        println!("attempting to load prediction cache...");
        let cache_result = File::open(path).ok().and_then(|file| bincode::deserialize_from(file).ok());

        if let Some(result) = cache_result {
            println!("... loaded cache");
            result
        }
        else {
            println!("... invalid, computing new prediction cache");
            type Count<T> = HashMap<T, usize>;
            type Chain<T> = HashMap<Vec<T>, Count<T>>;
            use string_interner::StringInterner;

            let mut strings: StringInterner<StringId> = string_interner::StringInterner::with_capacity(2048);

            // initialize the statistical data which we are going fill by analyzing the corpus
            let mut word_chains: Chain<StringId> = Chain::with_capacity(1024*1024);
            let mut all_chars: Count<char> = HashMap::new();
            let mut sentence_starters : Count<StringId> = HashMap::new();
            let mut all_words : Count<StringId> = HashMap::new();
            let mut word_count: u128 = 0;
            let mut char_count: u128 = 0;

            for string in corpus::sentences() {
                let sentence = split_to_words(&string);
                if sentence.is_empty() { continue; }

                let words: Vec<StringId> = sentence.iter().map(|string| strings.get_or_intern(string)).collect();

                for &word in &words {
                    *all_words.entry(word).or_insert(0) += 1;
                }

                *sentence_starters.entry(*words.first().unwrap()).or_insert(0) += 1;

                word_count += sentence.len() as u128;

                for char in string.chars() {
                    *all_chars.entry(char).or_insert(0) += 1;
                    char_count += 1;
                }

                for chain_len in 1 ..=MAX_CHAIN_LEN {
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
            println!("processed {} chars", char_count);
            println!("collected {} distinct words", strings.len());
            println!("collected {} prediction entries", word_chains.len());


            fn map_to_sorted_count_vec<T>(map: impl Iterator<Item = (T, usize)>) -> Vec<(usize, T)> {
                let tree: BTreeMap<usize, T> = map.map(|(value, count)| (count, value)).collect();
                tree.into_iter().rev().collect()
            }

            fn map_to_sorted_vec<T>(map: Count<T>) -> Vec<T> {
                let tree: BTreeMap<usize, T> = map.into_iter()
                    .map(|(value, count)| (count, value)).collect();

                tree.into_iter().map(|(_, value)| value).rev().collect()
            }

            let words = map_to_sorted_vec(all_words);
            let top_word_count = 7;

            println!("top {} common words: {:?}", top_word_count, words[..top_word_count].iter().map(|&id| strings.resolve(id).unwrap()).collect::<Vec<_>>());

            let starters = map_to_sorted_vec(sentence_starters);
            let starters: Vec<String> = starters.into_iter()
                .filter(|starter| !words[..top_word_count].contains(starter))
                .map(|id| strings.resolve(id).unwrap().to_string())
                .collect();

            let chains: HashMap<Vec<StringId>, Vec<StringId>> = word_chains
                .into_iter().filter(|(key, values)| !values.is_empty() && (key.len() == 1 || values.len() > 1))
                .map(|(words, successors)| (words, map_to_sorted_vec(successors)))
                .collect();

            println!("condensed to {} prediction entries", chains.len());

            let top_words = words[.. top_word_count].to_vec();


            let result = (starters, strings, chains, top_words);
            bincode::serialize_into(File::create(path).unwrap(), &result).unwrap();
            result
        }
    };

    move |previous_words: &[String]| -> Vec<String> {
        if previous_words.is_empty() { return starters.clone(); }

        (1 ..= MAX_CHAIN_LEN.min(previous_words.len())).rev().flat_map(|chain_len| {
            let sub_key_words = &previous_words[previous_words.len() - chain_len .. ];
            println!("sub key: {:?}", sub_key_words);

            let key_words: Vec<StringId> = sub_key_words.iter()
                .flat_map(|string| strings.get(string))
                .collect();

            let options = chains.get(&key_words);
            options.into_iter().flatten()
                .filter(|&id| !top_words.contains(id))
                .map(|&id| strings.resolve(id).unwrap().to_owned())

        }).take(7).collect()
    }
}

pub fn _gpt2_predictor() -> impl (Fn(&str) -> Vec<(Option<String>, Vec<String>)>) {
    use rust_bert::pipelines::generation::{GPT2Generator, LanguageGenerator, GenerateConfig};
    use rust_bert::gpt2::*;
    use rust_bert::resources::{Resource, RemoteResource};

    let max_base_word_count = 4;

    // create the GPT-2 Model that generates our variations
    let model = GPT2Generator::new(GenerateConfig {

        // vary length from 2 to 10 to keep it short
        min_length: 1,
        max_length: max_base_word_count as u64 + 2, // cannot be 1 because it includes our prefix

        // always compute four variations at once
        num_return_sequences: 4,

        do_sample: true, // no random funny business
        temperature: 2.5,
        // num_beams: 4,

        model_resource: Resource::Remote(RemoteResource::from_pretrained(Gpt2ModelResources::GPT2_MEDIUM)),
        merges_resource: Resource::Remote(RemoteResource::from_pretrained(Gpt2MergesResources::GPT2_MEDIUM)),
        vocab_resource: Resource::Remote(RemoteResource::from_pretrained(Gpt2VocabResources::GPT2_MEDIUM)),
        config_resource: Resource::Remote(RemoteResource::from_pretrained(Gpt2ConfigResources::GPT2_MEDIUM)),

        // device: Device::Cuda(0), // TODO

        ..Default::default()
    }).unwrap();

    let model = std::sync::Arc::new(std::sync::Mutex::new(model));

    move |base| {
        // generate a few predictions at once, using the GTP-2 generator
        println!("generating gpt-2 variations for \"{}\"", base);
        debug_assert!(split_to_words(base).len() <= max_base_word_count);

        model.lock().unwrap()
            .generate(if !base.trim().is_empty() { Some(vec![base]) } else { None }, None).into_iter()
            .map(|prediction|{
                println!("gpt output: {}", prediction.trim_end());

                // remove the first few words which we gave the predictor
                let predictions = &prediction[base.len() ..];
                let mut words = split_to_words(predictions);

                if !predictions.starts_with(char::is_whitespace) {
                    let completion = words.remove(0);
                    (Some(completion), words)
                }
                else {
                    (None, words)
                }

            }).collect()
    }
}


// use std::collections::{HashMap, BTreeMap, HashSet};
//
// use string_interner::StringInterner;
// type StringId = string_interner::Sym;
//
// use rayon::iter::{IntoParallelIterator, ParallelIterator};
// // use crate::correction::char_vec;
//
//
// fn main() {
//
//     type Count<T> = HashMap<T, usize>;
//     type Chain<T> = HashMap<Vec<T>, Count<T>>;
//     let max_chain_len = 1;
//
//     let mut strings: StringInterner<StringId> = string_interner::StringInterner::with_capacity(2048);
//
//     // initialize the statistical data which we are going to analyze
//     let mut word_chains: Chain<StringId> = Chain::with_capacity(1024*1024);
//     let mut sentence_starter_words: Count<StringId> = HashMap::with_capacity(1024*1024);
//     let mut sentence_starter_chars: Count<char> = HashMap::new();
//     let mut word_starter_chars: Count<char> = HashMap::new();
//     let mut all_chars: Count<char> = HashMap::new();
//     let mut all_words : Count<StringId> = HashMap::new();
//     let mut word_count: u128 = 0;
//     let mut char_count: u128 = 0;
//
//     // synchroneously collect all the parsed data into our statistical hashmap
//     for (string, sentence) in sentences {
//         let words: Vec<StringId> = sentence.iter().map(|string| strings.get_or_intern(string)).collect();
//
//         for &word in &words {
//             *all_words.entry(word).or_insert(0) += 1;
//         }
//
//         word_count += sentence.len() as u128;
//
//         *sentence_starter_words.entry(*words.first().unwrap()).or_insert(0) += 1;
//         *sentence_starter_chars.entry(sentence.first().unwrap().chars().next().unwrap()).or_insert(0) += 1;
//
//         for char in string.chars() {
//             *all_chars.entry(char).or_insert(0) += 1;
//             char_count += 1;
//         }
//
//         for word in &sentence {
//             *word_starter_chars.entry(word.chars().next().unwrap()).or_insert(0) += 1;
//
//             // for letter in word.chars() {
//             //     *word_chars.entry(letter).or_insert(0) += 1;
//             // }
//         }
//
//         /*for chain_len in 1 ..= max_chain_len {
//             for key in words.windows(chain_len + 1) {
//                 let value = &key[chain_len];
//                 let key = Vec::from(&key[ .. chain_len]);
//
//                 let map = word_chains.entry(key).or_insert_with(HashMap::new);
//                 *map.entry(*value).or_insert(0) += 1;
//             }
//         }*/
//
//         println!("processed {} words", word_count);
//     }
//
//     println!("analyzed all files");
//     println!("processed {} words", word_count);
//     println!("processed {} chars", char_count);
//     println!("collected {} distinct words", strings.len());
//     println!("collected {} prediction entries", word_chains.len());
//
//
//     fn map_to_sorted_count_vec<T>(map: impl Iterator<Item = (T, usize)>) -> Vec<(usize, T)> {
//         let tree: BTreeMap<usize, T> = map.map(|(value, count)| (count, value)).collect();
//         tree.into_iter().rev().collect()
//     }
//
//     fn map_to_sorted_vec<T>(map: Count<T>) -> Vec<T> {
//         let tree: BTreeMap<usize, T> = map.into_iter()
//             .map(|(value, count)| (count, value)).collect();
//
//         tree.into_iter().map(|(_, value)| value).rev().collect()
//     }
//
//     let words = map_to_sorted_count_vec(
//         all_words.into_iter()
//             .map(|(key, count)| (strings.resolve(key).unwrap().to_string(), count))
//     );
//
//     let chars = map_to_sorted_count_vec(
//         all_chars.iter().map(|(&char, &count)| (char.to_string(), count))
//     );
//
//     fs::write("results/words.txt", format!("{:#?}", words)).unwrap();
//     fs::write("results/chars.txt", format!("{:#?}", chars)).unwrap();
//
//     fs::write("results/corpus.txt", format!(
//         "words: {}, chars:{}, distinct words: {}",
//         word_count, char_count, strings.len(),
//     )).unwrap();
//
//     let starter_words = map_to_sorted_vec(sentence_starter_words);
//     let starter_word_strings: Vec<&str> = starter_words.iter().map(|&id| strings.resolve(id).unwrap()).collect();
//
//     let chains: HashMap<Vec<StringId>, Vec<StringId>> = word_chains
//         .into_par_iter().filter(|(key, values)| !values.is_empty() && (key.len() == 1 || values.len() > 1))
//         .map(|(words, successors)| (words, map_to_sorted_vec(successors)))
//         .collect();
//
//     println!("condensed to {} prediction entries", chains.len());
//     println!();
//
//     println!("generic statistics: ");
//     println!("\tall chars: {:#?}", map_to_sorted_count_vec(all_chars.into_iter()));
//     println!("\tword starter chars: {:#?}", map_to_sorted_count_vec(word_starter_chars.into_iter()));
//     println!("\tsentence starter chars: {:#?}", map_to_sorted_count_vec(sentence_starter_chars.into_iter()));
//
//     println!();
//     println!("you type, i predict.");
//     println!("why not start with one of these words: {}?", &starter_word_strings[..24.min(starter_words.len())].join(", "));
//     println!("type something!");
//
//
//     let predict = |word_ids: &[StringId]| -> Vec<Vec<StringId>> {
//         for chain_len in (1 ..= max_chain_len.min(word_ids.len())).rev() {
//             let key: Vec<StringId> = Vec::from(&word_ids[word_ids.len() - chain_len .. ]);
//             // println!("prediction key: {:?}", key);
//
//             let options = chains.get(&key).map(|options|{
//                 &options[ .. options.len().min(20) ]
//             });
//
//             // println!("first prediction: {:?}", options);
//
//             if let Some(options) = options {
//                 return options.iter().map(|&option| {
//                     let mut key = key.clone();
//                     key.push(option);
//
//                     if let Some(&option2) = chains.get(&key).and_then(|successors| successors.first()) {
//                         // println!("{} {}", strings.resolve(option).unwrap(), strings.resolve(option2).unwrap());
//                         vec![ option, option2 ]
//                     }
//                     else {
//                         key.remove(0);
//                         if let Some(&option2) = chains.get(&key).and_then(|successors| successors.first()) {
//                             // println!("{} ({}?)", strings.resolve(option).unwrap(), strings.resolve(option2).unwrap());
//                             vec![ option, option2 ]
//                         }
//                         else {
//                             // println!("{}", strings.resolve(option).unwrap());
//                             vec![ option ]
//                         }
//                     }
//                 }).collect()
//             }
//         };
//
//         Vec::new()
//     };
//
//
//
//     loop {
//         let input = {
//             let mut input = String::new();
//             io::stdin().read_line(&mut input).unwrap();
//             input.trim().to_string()
//         };
//
//         let words = split_to_words(&input);
//         println!("words: {:?}", words);
//
//         let mut word_ids: Vec<StringId> = words.iter().map(|string| strings.get_or_intern(string)).collect();
//
//         // for word_index in 0..word_ids.len() {
//         // let as_vec = char_vec(strings.resolve(word_ids[word_index]).unwrap());
//         // let corrections = correction::tier1_variations(&as_vec);
//         // let words_before_that = &word_ids[..word_index];
//
//         // let predictions: HashSet<StringId> = predict(words_before_that).into_iter()
//         //     .map(|mut vec| vec.remove(0)).collect();
//
//         // only correct where the corrected word would also be predicted
//         // let correction = corrections.filter_map(|string| strings.get(string))
//         //     .filter(|correction| predictions.contains(&correction))
//         //     .next();
//         //
//         // if let Some(correction) = correction {
//         //     word_ids[word_index] = correction;
//         // }
//         // }
//
//         let user_words: Vec<&str> = word_ids.iter().map(|&id| strings.resolve(id).unwrap()).collect();
//         println!("corrected words: {:?}", user_words);
//
//         let predictions = predict(word_ids.as_slice());
//         // println!("actual predictions: {:?}", predictions);
//
//         for predicted_words in predictions {
//             let predicted_words: Vec<&str> = predicted_words.into_iter()
//                 .map(|id| strings.resolve(id).unwrap()).collect();
//
//             println!("\t{} {}", user_words.join(" "), predicted_words.join(" ")); // TODO base selection on prediction of last word!
//         }
//     }
// }
//
