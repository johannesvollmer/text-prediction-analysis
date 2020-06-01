// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt


mod corpus;
mod completion;

use crate::corpus::split_to_words;

fn main() {
    use serde::{Deserialize, Serialize};
    use std::sync::{Mutex};

    let trie = completion::build(corpus::words());
    let trie = Mutex::new(trie);

    println!("listening on localhost:3000");
    rouille::start_server_with_pool("localhost:3000", Some(1), move |request| {

        #[derive(Serialize, Deserialize)]
        struct Request {
            previous: String,
            next: String
        }

        #[derive(Serialize, Deserialize)]
        struct Response {
            completions: Vec<String>,
        }

        let request: Request = serde_json::from_reader(request.data().unwrap()).unwrap();

        let candidates: Vec<String> = {
            let pre_words = split_to_words(&request.previous);

            let mut completions: Vec<(String, usize)> = trie.lock().unwrap()
                .iter_prefix(pre_words.last().unwrap().as_bytes())
                .map(|(word, &count)| (String::from_utf8(word).unwrap(), count)).collect();

            completions.sort_by_key(|(_, count)| *count);
            completions.into_iter().map(|(word, _)| word).collect()
        };

        rouille::Response::json(&Response { completions: candidates })
    });
}
