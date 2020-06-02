

// macro_rules! concat_iter {
//     ($iter1: expr) => { $iter1 };
//
//     ($iter1: expr, $iter2: expr) => {{
//         let (mut i1, mut i2) = ($iter1, $iter2); // make variable mutable
//         std::iter::from_fn(move || i1.next().or_else(|| i2.next()))
//     }};
//
//     ($iter1: expr, $( $rest: expr),* ) => {
//         concat_iter!($iter1, concat_iter!($($rest),*))
//     };
// }

pub fn char_vec(string: &str) -> Vec<char> {
    string.chars().collect()
}

pub fn tier2_only_variations<'s>(string: &'s [char]) -> impl Iterator<Item = String> + 's {
    tier1_variations(string)
        .flat_map(|variation| tier1_variations(&char_vec(&variation)))
}

pub fn tier1_variations(word: &[char]) -> impl Iterator<Item = String> {
    const CHARS: &'static str = "abcdefghijklmnopqrstuvwxyz";

    // variations that have one of the chars deleted
    let deletes = (0..word.len()).map(|delete_at|{
        let slice = [ &word[..delete_at], &word[delete_at + 1..] ];
        slice.iter().flat_map(|slice| slice.into_iter()).collect()
    });

    // variations that have two neighbouring chars swapped
    let swaps = (0..word.len() - 1).map(|left_idx|{
        let slice = [
            &word[..left_idx],
            &[word[left_idx + 1], word[left_idx]],
            &word[left_idx + 2..]
        ];

        slice.iter().flat_map(|slice| slice.into_iter()).collect()
    });

    // variations that have char replaced
    let replaces = (0..word.len()).flat_map(|insert_at|{
        CHARS.chars().map(move |insert_letter|{
            let slice = [ &word[..insert_at], &[insert_letter], &word[insert_at + 1..] ];
            slice.iter().flat_map(|slice| slice.into_iter()).collect()
        })
    });

    // variations that have an additional char inserted
    let inserts = (0..=word.len()).flat_map(|insert_at|{
        CHARS.chars().map(move |insert_letter|{
            let slice = [ &word[..insert_at], &[insert_letter], &word[insert_at..] ];
            slice.iter().flat_map(|slice| slice.into_iter()).collect()
        })
    });

    deletes.chain(swaps).chain(replaces).chain(inserts)
        .collect::<Vec<String>>().into_iter()
    // concat_iter!(deletes, swaps, replaces, inserts)
}
