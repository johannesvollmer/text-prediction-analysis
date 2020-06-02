
/// Return a lambda that returns a list of completions based on a word fragment.
pub fn build(words: impl Iterator<Item = String>)
    -> impl (Fn(&str) -> Vec<String>)
{

    pub use patricia_tree::PatriciaSet;
    use patricia_tree::PatriciaMap;

    let mut map = PatriciaMap::new();

    for word in words {
        let count = map.get(&word).unwrap_or(&0) + 1;
        map.insert(&word, count);
    }

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


