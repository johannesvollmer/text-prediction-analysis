// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt


mod corpus;
mod completion;
mod server;
mod prediction;
mod correction;

use crate::corpus::split_to_words;
use crate::server::{Response, Request};
use std::io::Write;
use crate::correction::{char_vec, tier1_variations, tier2_only_variations};


fn main() {
    let respond = responder();

    let _ = respond(Request {
        previous: "hi thr".to_string(),
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
        let last_word = previous_words.last().cloned().unwrap_or(String::new());

        println!("requesting suggestions for word: {:?}", previous_words);

        let mut predicted_completions: Vec<String> = predict(&previous_words[ .. previous_words.len() - 1 ]);
        println!("unfiltered predicted based on all but the last word: {:?}", predicted_completions);

        predicted_completions.retain(|word| {
            word.starts_with(&last_word) || levenshtein::levenshtein(
                &last_word, &word[.. word.len().min(last_word.len())]
            ) < 3
        });

        println!("filtered predicted based on all but the last word: {:?}", predicted_completions);

        let mut char_completions: Vec<String> = complete(&last_word).into_iter().take(7).collect();
        println!("char completions: {:?}", char_completions);

        if char_completions.len() < 7 && last_word.len() > 2 {
            let lev_completions: Vec<String> = tier1_variations(&char_vec(&last_word))
                .flat_map(|prefix| complete(&prefix).first().cloned().into_iter())
                .take(7).collect();

            println!("lev1 completions: {:?}", lev_completions);
            char_completions.extend(lev_completions.into_iter())
        }

        if char_completions.len() < 7 && last_word.len() > 3 {
            let lev_completions: Vec<String> = tier2_only_variations(&char_vec(&last_word))
                .flat_map(|prefix| complete(&prefix).first().cloned().into_iter())
                .take(7).collect();

            println!("lev2 completions: {:?}", lev_completions);
            char_completions.extend(lev_completions.into_iter())
        }

        let mut completions = predicted_completions;
        completions.extend_from_slice(&char_completions);
        println!("all completions: {:?}", completions);

        let mut predicted_previous = previous_words.clone();
        predicted_previous.pop();
        predicted_previous.push(completions[0].clone()); // TODO predict for multiple top candidates!
        let predictions: Vec<String> = predict(&predicted_previous);
        println!("char-completed predictions: {:?}", predictions);

        let response = Response { completions: completions, predictions: predictions };
        response
    };

    respond
}