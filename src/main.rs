// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt


mod corpus;
mod completion;
mod server;
mod prediction;

use crate::corpus::split_to_words;
use crate::server::Response;

fn main() {
    println!("analyzing text...");

    // analyze the text corpora
    let complete = completion::build(corpus::words());

    let predict = prediction::predictor();

    println!("analysis finished");

    // serve the computed completitions
    server::run(move |request| {
        let previous_words = split_to_words(&request.previous);

        let completions: Vec<String> = complete(&previous_words.last().unwrap()).into_iter()
            .filter(|(_, count)| *count > 2)
            .map(|(word, _)| word).collect();

        let predictions: Vec<String> = predict(&request.previous);

        Response { completions, predictions }
    });
}
