use crate::corpus;
use std::path::Path;
use std::fs::File;
use std::iter::FromIterator;

/// Return a lambda that returns a list of completions based on a word fragment.
pub fn build() -> impl (Fn(&str) -> Vec<String>) {
    pub use patricia_tree::PatriciaSet;
    use patricia_tree::PatriciaMap;

    let path = Path::new(".completion-cache");
    let map = {
        println!("attempting to load completion cache...");
        let cache_result: Option<Vec<(Vec<u8>, usize)>> = File::open(path).ok().and_then(|file| bincode::deserialize_from(file).ok());

        if let Some(result) = cache_result {
            println!("... loaded cache");
            PatriciaMap::from_iter(result.into_iter())
        }
        else {
            println!("... invalid, computing new prediction cache");
            let mut map = PatriciaMap::new();

            for word in corpus::words() {
                let count = map.get(&word).unwrap_or(&0) + 1;
                map.insert(&word, count);
            }

            bincode::serialize_into(
                File::create(path).unwrap(),
                &map.clone().into_iter().collect::<Vec<(Vec<u8>, usize)>>()
            ).unwrap();

            map
        }
    };

    move |previous_word|{
        let mut completions: Vec<(String, usize)> = map
            .iter_prefix(previous_word.as_bytes())
            .map(|(word, &count)| (String::from_utf8(word).unwrap(), count)).collect();

        // sort the completions by number of occurrences in the corpus (best at last)
        completions.sort_by_key(|(_, count)| *count);

        completions.into_iter().rev()
            .filter(|(_, count)| *count > 3)
            .map(|(word, _)| word)
            .collect()
    }
}


