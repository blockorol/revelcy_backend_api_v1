use anyhow::{Context, Result};
use crate::models::premarket::SolanaNetwork;
use super::env::read_revelcy_auth;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::Transaction;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bincode::{serialize as bincode_serialize, deserialize as bincode_deserialize};


pub fn sign_tx_with_revelcy(
    tx_base64: &str,
    network: SolanaNetwork,
    extra_signers: Option<&[Keypair]>,
) -> Result<String> {
    let revelcy = read_revelcy_auth(network);

    // 1) Декодим вход (уже частично/полностью подписанную пользователем транзу)
    let raw = BASE64
        .decode(tx_base64)
        .context("BASE64 decode(tx_base64) failed")?;
    let mut tx: Transaction =
        bincode_deserialize(&raw).context("bincode deserialize(Transaction) failed")?;

    let blockhash = tx.message.recent_blockhash;

    // 2) Собираем список всех подписантов: revelcy + (опционально) доп. ключи
    let mut signers: Vec<&Keypair> = Vec::new();
    signers.push(&revelcy);

    if let Some(extra) = extra_signers {
        for kp in extra {
            signers.push(kp);
        }
    }

    // 3) Частичная подпись всеми ключами
    tx.try_partial_sign(&signers, blockhash)
        .context("failed to partially sign transaction with revelcyAuth and extra_signers")?;

    // 4) Сериализация обратно в base64
    let signed_raw =
        bincode_serialize(&tx).context("bincode serialize(signed Transaction) failed")?;
    let signed_b64 = BASE64.encode(signed_raw);

    Ok(signed_b64)
}
