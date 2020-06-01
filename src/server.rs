// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub previous: String,
    pub next: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response {
    pub completions: Vec<String>,
}

/// Start a server that answers JSON suggestion requests with JSON suggestions.
pub fn run <F: (Fn(Request) -> Response)> (suggest: F)
    where F: Sync + Send + 'static
{
    println!("starting the suggestion server on localhost:3000");

    rouille::start_server_with_pool("localhost:3000", Some(1), move |request| {
        let request: Request = serde_json::from_reader(request.data().unwrap()).unwrap();
        println!("received a prediction request: {:?}", request);

        let answer = suggest(request);
        println!("computed answer: {:#?}", answer);

        rouille::Response::json(&answer)
    });
}
