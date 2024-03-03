mod swap;

use crate::swap::{create_swap, execute_swap};

use boltz_client::network::Chain;
use rocket::{get, launch, routes, Config, State};
use serde_json::json;
use threadpool::ThreadPool;

#[get("/.well-known/lnurlp/liquid")]
fn index() -> &'static str {
    r#"{
    "callback": "https://zzd.es/.well-known/lnurlp/liquid/callback",
    "maxSendable": 10000000,
    "minSendable": 1000000,
    "metadata": "[[\"text/plain\",\"Swap sats to liquid for Andrei\"]]",
    "tag": "payRequest"
}"#
}

#[get("/.well-known/lnurlp/liquid/callback?<amount>")]
fn callback(amount: u64, pool: &State<ThreadPool>) -> String {
    dbg!(amount);
    let to_address = env!("TO_ADDRESS").to_string();
    let swap = create_swap(amount / 1000, to_address, Chain::Liquid).unwrap();
    let invoice = swap.response.invoice.clone().unwrap();

    pool.execute(|| {
        if let Err(e) = execute_swap(swap) {
            eprintln!("{e:?}");
        }
    });

    json!({
        "pr": invoice,
        "routes": Vec::<()>::new(),
    })
    .to_string()
}

#[get("/.well-known/lnurlp/hybrid")]
fn index_hybrid() -> &'static str {
    r#"{
    "callback": "https://zzd.es/.well-known/lnurlp/hybrid/callback",
    "maxSendable": 1000000000,
    "minSendable": 1000,
    "metadata": "[[\"text/plain\",\"Hybrid address of Andrei\"]]",
    "tag": "payRequest"
}"#
}

#[get("/.well-known/lnurlp/hybrid/callback?<amount>")]
fn callback_hybrid(amount: u64, pool: &State<ThreadPool>) -> String {
    dbg!(amount);
    if amount < 10_000_000 {
        let url = format!(
            "https://getalby.com/lnurlp/andrei21/callback?amount={}",
            amount
        );
        let response = ureq::get(&url).call().unwrap();
        if response.status() != 200 {
            panic!("getalby returnes status code: {}", response.status());
        }
        response.into_string().unwrap()
    } else {
        let to_address = env!("TO_ADDRESS").to_string();
        let swap = create_swap(amount / 1000, to_address, Chain::Liquid).unwrap();
        let invoice = swap.response.invoice.clone().unwrap();

        pool.execute(|| {
            if let Err(e) = execute_swap(swap) {
                eprintln!("{e:?}");
            }
        });

        json!({
            "pr": invoice,
            "routes": Vec::<()>::new(),
        })
        .to_string()
    }
}

#[launch]
fn rocket() -> _ {
    let config = Config {
        port: 8001,
        ..Config::default()
    };
    rocket::build()
        .configure(config)
        .manage(ThreadPool::new(8))
        .mount("/", routes![index, callback, index_hybrid, callback_hybrid])
}
