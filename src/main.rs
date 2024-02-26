mod swap;

use crate::swap::{create_swap, execute_swap};

use boltz_client::network::Chain;
use rocket::{get, launch, routes, Config, State};
use serde_json::json;
use threadpool::ThreadPool;

#[get("/.well-known/lnurlp/<username>")]
fn index(username: &str) -> String {
    match username {
		"andrei" => r#"{"status":"OK","tag":"payRequest","commentAllowed":255,"callback":"https://getalby.com/lnurlp/andrei21/callback","metadata":"[[\"text/identifier\",\"andrei@zzd.es\"],[\"text/plain\",\"Sats for andrei\"]]","minSendable":1000,"maxSendable":11000000000,"payerData":{"name":{"mandatory":false},"email":{"mandatory":false}}}"#.to_string(),
		"liquid" => {
			let callback = "https://zzd.es/.well-known/lnurlp/liquid/callback";
			let description = "Swap sats to liquid for Andrei";
			let r = r#"{
    "callback": "{callback}",
    "maxSendable": 10000000,
    "minSendable": 1000000,
    "metadata": "[[\"text/plain\",\"{description}\"]]",
    "tag": "payRequest"
}
"#;
			r.replacen("{callback}", callback, 1).replacen("{description}", description, 1)
		},
		_ => "not found".to_string(),
	}
}

#[get("/.well-known/lnurlp/liquid/callback?<amount>")]
fn callback(amount: u64, pool: &State<ThreadPool>) -> String {
    dbg!(amount);
    let to_address =
        "VJLJPujJ83ZCH4sPEsASXfka82JmN33T6DS8uBKQtnnWwbJtXtuK2t6tJwsauSF8jSUtuStjFv4JPNsV"
            .to_string();
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

#[launch]
fn rocket() -> _ {
    let pool = ThreadPool::new(8);
    let config = Config {
        port: 8001,
        ..Config::default()
    };
    rocket::build()
        .configure(config)
        .manage(pool)
        .mount("/", routes![index, callback])
}

fn _foo_main() {
    let pool = ThreadPool::new(8);

    // Testnet
    let to_address = "tlq1qqv4z28utgwunvn62s3aw0qjuw3sqgfdq6q8r8fesnawwnuctl70kdyedxw6tmxgqpq83x6ldsyr4n6cj0dm875k8g9k85w2s7".to_string();
    let swap = create_swap(10_000, to_address, Chain::LiquidTestnet).unwrap();
    println!(
        "    **** Pay Invoice: {}",
        swap.response.get_invoice().unwrap()
    );
    pool.execute(|| {
        if let Err(e) = execute_swap(swap) {
            eprintln!("{e:?}");
        }
    });

    // Mainnet
    //	let to_address = "VJLJPujJ83ZCH4sPEsASXfka82JmN33T6DS8uBKQtnnWwbJtXtuK2t6tJwsauSF8jSUtuStjFv4JPNsV".to_string();
    //    let r = foo(1_000, to_address, Chain::Liquid);

    pool.join();
}
