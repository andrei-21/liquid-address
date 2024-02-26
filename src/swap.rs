use boltz_client::error::Error;
use boltz_client::network::electrum::ElectrumConfig;
use boltz_client::network::Chain;
use boltz_client::swaps::boltz::{
    BoltzApiClient, CreateSwapRequest, CreateSwapResponse, BOLTZ_MAINNET_URL, BOLTZ_TESTNET_URL,
};
use boltz_client::swaps::liquid::LBtcSwapTx;
use boltz_client::util::secrets::{LBtcReverseRecovery, LiquidSwapKey, Preimage, SwapKey};
use boltz_client::ZKKeyPair;
use serde_json::Value;
use std::str::FromStr;
use std::thread;
use std::time::Duration;

const BOLTZ_V2_MAINNET_URL: &str = "https://api.boltz.exchange/v2";
const BOLTZ_V2_TESTNET_URL: &str = "https://api.testnet.boltz.exchange/v2";

pub struct CreatedSwap {
    pub response: CreateSwapResponse,
    preimage: Preimage,
    claim_key_pair: ZKKeyPair,
    claim_fee: u64,
    to_address: String,
    chain: Chain,
}

pub fn create_swap(amount: u64, to_address: String, chain: Chain) -> Result<CreatedSwap, Error> {
    let boltz_url = match chain {
        Chain::Liquid => BOLTZ_MAINNET_URL,
        Chain::LiquidTestnet => BOLTZ_TESTNET_URL,
        _ => panic!("Unsupported chain"),
    };
    let client = BoltzApiClient::new(boltz_url);

    let pairs = client.get_pairs()?;
    let lbtc_pair = pairs
        .get_lbtc_pair()
        .ok_or(Error::Protocol("Missing L-BTC pair".to_string()))?;

    let invoice_amount = amount;
    //    let base_fees = lbtc_pair.fees.reverse_boltz(invoice_amount);
    //    let reverse_lockup = lbtc_pair.fees.reverse_lockup();
    let claim_fee = lbtc_pair.fees.reverse_claim_estimate();
    // println!("   CALCULATED FEES: {base_fees}");
    // println!("    REVERSE_LOCKUP: {reverse_lockup}");
    // println!("         CLAIM FEE: {claim_fee}");
    // println!(
    //     "    ONCHAIN LOCKUP: {}",
    //     invoice_amount - base_fees - reverse_lockup
    // );
    // println!(
    //     "ONCHAIN RECIEVABLE: {}",
    //     invoice_amount - base_fees - claim_fee - reverse_lockup
    // );

    let mnemonic = env!("MNEMONIC");
    let swap_key = SwapKey::from_reverse_account(mnemonic, "", chain, 1)?;
    let lsk: LiquidSwapKey = LiquidSwapKey::try_from(swap_key)?;
    let claim_key_pair = lsk.keypair;
    let claim_pubkey = claim_key_pair.public_key().to_string();
    let preimage = Preimage::new();
    let hash = preimage.sha256.to_string();

    let request = CreateSwapRequest::new_lbtc_reverse_invoice_amt(
        lbtc_pair.hash,
        hash,
        claim_pubkey,
        invoice_amount,
    );
    let response = client.create_swap(request)?;
    println!("Response: {response:?}");
    // println!("Onchain Amount: {:?}", response.onchain_amount);
    // println!("Invoice: {}", response.get_invoice()?);

    let id = response.get_id();
    let blinding_key = ZKKeyPair::from_str(&response.get_blinding_key()?)?;
    let boltz_script_elements =
        response.into_lbtc_rev_swap_script(&preimage, &claim_key_pair, chain)?;

    let recovery = LBtcReverseRecovery::new(
        &id,
        &preimage,
        &claim_key_pair,
        &blinding_key,
        &response.get_redeem_script()?,
    );
    println!("RECOVERY: {:#?}", recovery);
    println!("timeoutBlockHeight: {}", response.get_timeout()?);
    println!("nLocktime: {}", boltz_script_elements.locktime);

    Ok(CreatedSwap {
        response,
        preimage,
        claim_key_pair,
        claim_fee,
        to_address,
        chain,
    })
}

pub fn execute_swap(swap: CreatedSwap) -> Result<(), Error> {
    let boltz_v2_url = match swap.chain {
        Chain::Liquid => BOLTZ_V2_MAINNET_URL,
        Chain::LiquidTestnet => BOLTZ_V2_TESTNET_URL,
        _ => panic!("Unsupported chain"),
    };
    let network_config = match swap.chain {
        Chain::Liquid => ElectrumConfig::new(swap.chain, "blockstream.info:995", true, true, 10),
        Chain::LiquidTestnet => ElectrumConfig::default_liquid(),
        _ => panic!("Unsupported chain"),
    };
    loop {
        thread::sleep(Duration::from_secs(4));
        let status = query_swap_status(boltz_v2_url, &swap.response.id)?;
        println!("Swap status: {status}");
        if status == "transaction.mempool" || status == "transaction.confirmed" {
            let boltz_script_elements = swap.response.into_lbtc_rev_swap_script(
                &swap.preimage,
                &swap.claim_key_pair,
                swap.chain,
            )?;
            let rev_swap_tx = LBtcSwapTx::new_claim(
                boltz_script_elements,
                swap.to_address.clone(),
                &network_config,
            );
            let rev_swap_tx = match rev_swap_tx {
                Ok(rev_swap_tx) => rev_swap_tx,
                Err(e) => {
                    println!("Failed to create claim: {e:?}");
                    continue;
                }
            };
            let signed_tx =
                rev_swap_tx.sign_claim(&swap.claim_key_pair, &swap.preimage, swap.claim_fee)?;
            let txid = rev_swap_tx.broadcast(signed_tx, &network_config)?;
            println!("Claim txid: {txid}");
            break;
        }
    }

    Ok(())
}

fn query_swap_status(boltz_url: &str, swap_id: &str) -> Result<String, ureq::Error> {
    let url = format!("{boltz_url}/swap/{swap_id}");
    let response = ureq::get(&url).call()?;
    if response.status() == 400 {
        return Ok("Not found".to_string());
    }
    let json = response.into_string()?;
    let v: Value = serde_json::from_str(&json).unwrap();
    let status = v
        .get("status")
        .unwrap()
        .as_str()
        .unwrap_or_default()
        .to_string();
    Ok(status)
}
