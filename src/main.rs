mod swap;

use crate::swap::{create_swap, execute_swap};

use boltz_client::network::Chain;
use threadpool::ThreadPool;

#[tokio::main]
async fn main() {
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
