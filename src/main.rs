// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt


mod corpus;
mod completion;
mod server;

use crate::corpus::split_to_words;
use crate::server::Response;

fn main() {
    // analyze the text corpora
    let complete = completion::build(corpus::words());

    // serve the computed completitions
    server::run(move |request| {
        let previous_words = split_to_words(&request.previous);

        let completions: Vec<String> = complete(&previous_words.last().unwrap())
            .into_iter().map(|(word, _)| word).collect();

        Response { completions }
    });
}
