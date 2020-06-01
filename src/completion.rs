

pub use patricia_tree::PatriciaSet;
use patricia_tree::PatriciaMap;

pub fn build(words: impl Iterator<Item = String>) -> PatriciaMap<usize> {
    let mut map = PatriciaMap::new();
    for word in words {
        let count = map.get(&word).unwrap_or(&0) + 1;
        map.insert(&word, count);
    }

    map
}


