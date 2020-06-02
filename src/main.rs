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
use crate::server::{Response, Request};
use std::io::Write;


fn main() {
    let respond = responder();

    let _ = respond(Request {
        previous: "hi wo".to_string(),
        next: "".to_string()
    });

    // start a server that returns suggestions
    // server::run(respond());
}

fn responder() -> impl Fn(Request) -> Response {
    print!("preparing data bases...");
    std::io::stdout().flush().unwrap();


    let complete = completion::build(corpus::words());
    let predict = prediction::ngram_predictor(corpus::sentences());

    println!("... done.");

    let respond = move |request: Request|{
        let mut previous_words = split_to_words(&request.previous);
        println!("requesting suggestions for word: {:?}", previous_words);

        let mut predicted_completions: Vec<String> = predict(&previous_words[ .. previous_words.len() - 1 ]);
        println!("unfiltered predicted based on all but the last word: {:?}", predicted_completions);

        predicted_completions.retain(|word| word.starts_with(previous_words.last().unwrap()));
        println!("filtered predicted based on all but the last word: {:?}", predicted_completions);

        let char_completions: Vec<String> = complete(&previous_words.last().unwrap_or(&String::new()));
        println!("char completions: {:?}", char_completions);
        // char_completions.retain(|complete| predicted_completions.contains(complete));

        let mut completions = predicted_completions;
        completions.extend_from_slice(&char_completions);
        println!("all completions: {:?}", completions);

        let mut predicted_previous = previous_words.clone();
        predicted_previous.remove(0);
        predicted_previous.push(char_completions[0].clone()); // TODO predict for multiple top candidates!
        let predictions: Vec<String> = predict(&predicted_previous);
        println!("char-completed predictions: {:?}", predictions);

        let response = Response { completions: completions, predictions: predictions };
        response
    };

    respond
}