// TEXT CORPORA SOURCES (17.01.2020):
// http://www.anc.org/data/oanc/download/
// http://www.anc.org/data/masc/downloads/data-download/
// https://wortschatz.uni-leipzig.de/en/download/
// http://norvig.com/spell-correct.html -> http://norvig.com/big.txt

use serde::{Deserialize, Serialize};
use std::time::Instant as time;

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
pub fn run <F: (Fn(Request) -> Response)> (compute_suggestions: F)
    where F: Sync + Send + 'static
{
    println!("starting the suggestion server on localhost:3000");

    rouille::start_server_with_pool("localhost:3000", Some(1), move |request| {
        let request: Request = serde_json::from_reader(request.data().unwrap()).unwrap();

        println!("received a prediction request: {:?}", request);
        let start_time = time::now();

        let answer = compute_suggestions(request);

        let duration = (time::now() - start_time).as_secs_f32();
        println!("computed answer in {}s: {:#?}", duration, answer);
        rouille::Response::json(&answer)
    });
}
