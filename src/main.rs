// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt

mod corpus;
mod completion;

use crate::corpus::split_to_words;


fn main() {
    let trie = completion::build(corpus::words());

    loop {
        println!("type something!");

        let input = {
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();
            input.trim().to_string()
        };

        let words = split_to_words(&input);

        for (word, &count) in trie.iter_prefix(words.last().unwrap().as_bytes()) {
            if count > 2 {
                println!("\t{}: {}", String::from_utf8(word).unwrap(), count);
            }
        }
    }
}

