use rusqlite::{params, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

use super::{now_ts, Storage};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelPriceTierV2 {
    #[serde(alias = "min_input_tokens")]
    pub min_input_tokens: i64,
    #[serde(alias = "input_microusd_per_1m")]
    pub input_microusd_per_1m: i64,
    #[serde(alias = "cached_input_microusd_per_1m")]
    pub cached_input_microusd_per_1m: i64,
    #[serde(alias = "output_microusd_per_1m")]
    pub output_microusd_per_1m: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChargeComputationV2 {
    pub uncached_input_tokens: i64,
    pub numerator: i128,
    pub base_cost_microusd: i64,
    pub charged_cost_microusd: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChargeSnapshotInputV2 {
    pub request_log_id: i64,
    pub model_slug: String,
    pub usage_source: String,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub rate_multiplier_millis: i64,
    #[serde(default)]
    pub wallet_id: Option<String>,
    #[serde(default)]
    pub api_key_id: Option<String>,
    #[serde(default)]
    pub raw_usage_json: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChargeSnapshotV2 {
    pub request_log_id: i64,
    pub model_id: Option<String>,
    pub model_slug: String,
    pub tier_min_input_tokens: i64,
    pub usage_source: String,
    pub input_tokens: i64,
    pub cached_input_tokens: i64,
    pub output_tokens: i64,
    pub input_microusd_per_1m: i64,
    pub cached_input_microusd_per_1m: i64,
    pub output_microusd_per_1m: i64,
    pub rate_multiplier_millis: i64,
    pub base_cost_microusd: i64,
    pub charged_cost_microusd: i64,
    pub currency: String,
    pub created_at: i64,
}

fn checked_i64(value: i128, label: &str) -> Result<i64> {
    i64::try_from(value).map_err(|_| {
        rusqlite::Error::InvalidParameterName(format!("{label} exceeds SQLite INTEGER range"))
    })
}

fn ceil_div(value: i128, divisor: i128) -> i128 {
    if value <= 0 {
        0
    } else {
        (value + divisor - 1) / divisor
    }
}

pub fn compute_charge_v2(
    input_tokens: i64,
    cached_input_tokens: i64,
    output_tokens: i64,
    tier: &ModelPriceTierV2,
    rate_multiplier_millis: i64,
) -> Result<ChargeComputationV2> {
    if input_tokens < 0
        || cached_input_tokens < 0
        || output_tokens < 0
        || tier.input_microusd_per_1m < 0
        || tier.cached_input_microusd_per_1m < 0
        || tier.output_microusd_per_1m < 0
        || rate_multiplier_millis <= 0
    {
        return Err(rusqlite::Error::InvalidParameterName(
            "tokens, rates, and multiplier must be non-negative".to_string(),
        ));
    }
    let uncached_input_tokens = input_tokens.saturating_sub(cached_input_tokens).max(0);
    let cached_tokens_for_charge = cached_input_tokens.min(input_tokens);
    let input_part = i128::from(uncached_input_tokens)
        .checked_mul(i128::from(tier.input_microusd_per_1m))
        .ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("input charge overflow".to_string())
        })?;
    let cached_part = i128::from(cached_tokens_for_charge)
        .checked_mul(i128::from(tier.cached_input_microusd_per_1m))
        .ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("cached charge overflow".to_string())
        })?;
    let output_part = i128::from(output_tokens)
        .checked_mul(i128::from(tier.output_microusd_per_1m))
        .ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("output charge overflow".to_string())
        })?;
    let numerator = input_part
        .checked_add(cached_part)
        .and_then(|value| value.checked_add(output_part))
        .ok_or_else(|| rusqlite::Error::InvalidParameterName("charge overflow".to_string()))?;
    let charged_numerator = numerator
        .checked_mul(i128::from(rate_multiplier_millis))
        .ok_or_else(|| {
            rusqlite::Error::InvalidParameterName("multiplied charge overflow".to_string())
        })?;
    Ok(ChargeComputationV2 {
        uncached_input_tokens,
        numerator,
        base_cost_microusd: checked_i64(ceil_div(numerator, 1_000_000), "base cost")?,
        charged_cost_microusd: checked_i64(
            ceil_div(charged_numerator, 1_000_000_000),
            "charged cost",
        )?,
    })
}

fn map_snapshot(row: &rusqlite::Row<'_>) -> Result<ChargeSnapshotV2> {
    Ok(ChargeSnapshotV2 {
        request_log_id: row.get(0)?,
        model_id: row.get(1)?,
        model_slug: row.get(2)?,
        tier_min_input_tokens: row.get(3)?,
        usage_source: row.get(4)?,
        input_tokens: row.get(5)?,
        cached_input_tokens: row.get(6)?,
        output_tokens: row.get(7)?,
        input_microusd_per_1m: row.get(8)?,
        cached_input_microusd_per_1m: row.get(9)?,
        output_microusd_per_1m: row.get(10)?,
        rate_multiplier_millis: row.get(11)?,
        base_cost_microusd: row.get(12)?,
        charged_cost_microusd: row.get(13)?,
        currency: row.get(14)?,
        created_at: row.get(15)?,
    })
}

const SNAPSHOT_SELECT: &str = "SELECT request_log_id,model_id,model_slug,tier_min_input_tokens,
    usage_source,input_tokens,cached_input_tokens,output_tokens,input_microusd_per_1m,
    cached_input_microusd_per_1m,output_microusd_per_1m,rate_multiplier_millis,
    base_cost_microusd,charged_cost_microusd,currency,created_at
  FROM request_charge_snapshots";

impl Storage {
    pub fn select_model_price_tier_v2(
        &self,
        model_slug: &str,
        input_tokens: i64,
    ) -> Result<Option<(String, ModelPriceTierV2)>> {
        if input_tokens < 0 {
            return Err(rusqlite::Error::InvalidParameterName(
                "input tokens cannot be negative".to_string(),
            ));
        }
        self.conn
            .query_row(
                "SELECT m.id,t.min_input_tokens,t.input_microusd_per_1m,
                        t.cached_input_microusd_per_1m,t.output_microusd_per_1m
                 FROM models m
                 JOIN model_prices p ON p.model_id=m.id AND p.price_status<>'missing'
                 JOIN model_price_tiers t ON t.model_id=m.id AND t.min_input_tokens<=?2
                 WHERE m.slug=?1 COLLATE NOCASE
                 ORDER BY t.min_input_tokens DESC LIMIT 1",
                params![model_slug.trim(), input_tokens],
                |row| {
                    Ok((
                        row.get(0)?,
                        ModelPriceTierV2 {
                            min_input_tokens: row.get(1)?,
                            input_microusd_per_1m: row.get(2)?,
                            cached_input_microusd_per_1m: row.get(3)?,
                            output_microusd_per_1m: row.get(4)?,
                        },
                    ))
                },
            )
            .optional()
    }

    pub fn get_charge_snapshot_v2(&self, request_log_id: i64) -> Result<Option<ChargeSnapshotV2>> {
        self.conn
            .query_row(
                &format!("{SNAPSHOT_SELECT} WHERE request_log_id=?1"),
                [request_log_id],
                map_snapshot,
            )
            .optional()
    }

    pub fn record_charge_snapshot_v2(
        &self,
        input: &ChargeSnapshotInputV2,
    ) -> Result<ChargeSnapshotV2> {
        if !matches!(input.usage_source.as_str(), "actual" | "estimated") {
            return Err(rusqlite::Error::InvalidParameterName(
                "usage_source must be actual or estimated".to_string(),
            ));
        }
        let tx = self.conn.unchecked_transaction()?;
        if let Some(existing) = tx
            .query_row(
                &format!("{SNAPSHOT_SELECT} WHERE request_log_id=?1"),
                [input.request_log_id],
                map_snapshot,
            )
            .optional()?
        {
            tx.commit()?;
            return Ok(existing);
        }
        let price_status: Option<String> = tx
            .query_row(
                "SELECT p.price_status FROM models m JOIN model_prices p ON p.model_id=m.id
                 WHERE m.slug=?1 COLLATE NOCASE",
                [input.model_slug.trim()],
                |row| row.get(0),
            )
            .optional()?;
        match price_status.as_deref() {
            None => return Err(rusqlite::Error::QueryReturnedNoRows),
            Some("missing") => {
                return Err(rusqlite::Error::InvalidParameterName(
                    "model_price_missing".to_string(),
                ))
            }
            _ => {}
        }
        let (model_id, tier) = tx.query_row(
            "SELECT m.id,t.min_input_tokens,t.input_microusd_per_1m,
                    t.cached_input_microusd_per_1m,t.output_microusd_per_1m
             FROM models m JOIN model_price_tiers t ON t.model_id=m.id AND t.min_input_tokens<=?2
             WHERE m.slug=?1 COLLATE NOCASE ORDER BY t.min_input_tokens DESC LIMIT 1",
            params![input.model_slug.trim(), input.input_tokens],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    ModelPriceTierV2 {
                        min_input_tokens: row.get(1)?,
                        input_microusd_per_1m: row.get(2)?,
                        cached_input_microusd_per_1m: row.get(3)?,
                        output_microusd_per_1m: row.get(4)?,
                    },
                ))
            },
        )?;
        let computation = compute_charge_v2(
            input.input_tokens,
            input.cached_input_tokens,
            input.output_tokens,
            &tier,
            input.rate_multiplier_millis,
        )?;
        let now = now_ts();
        tx.execute(
            "INSERT INTO request_charge_snapshots(request_log_id,model_id,model_slug,
               tier_min_input_tokens,usage_source,input_tokens,cached_input_tokens,output_tokens,
               input_microusd_per_1m,cached_input_microusd_per_1m,output_microusd_per_1m,
               rate_multiplier_millis,base_cost_microusd,charged_cost_microusd,currency,created_at)
             VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,'USD',?15)",
            params![
                input.request_log_id,
                model_id,
                input.model_slug.trim(),
                tier.min_input_tokens,
                input.usage_source,
                input.input_tokens,
                input.cached_input_tokens,
                input.output_tokens,
                tier.input_microusd_per_1m,
                tier.cached_input_microusd_per_1m,
                tier.output_microusd_per_1m,
                input.rate_multiplier_millis,
                computation.base_cost_microusd,
                computation.charged_cost_microusd,
                now
            ],
        )?;
        if let Some(wallet_id) = input
            .wallet_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
        {
            let prior_ledger: Option<String> = tx
                .query_row(
                    "SELECT id FROM app_wallet_ledger_entries
                 WHERE request_log_id=?1 AND entry_kind='request_charge' LIMIT 1",
                    [input.request_log_id],
                    |row| row.get(0),
                )
                .optional()?;
            if prior_ledger.is_some() {
                return Err(rusqlite::Error::InvalidParameterName(
                    "request charge ledger exists without snapshot".to_string(),
                ));
            }
            let charge = computation.charged_cost_microusd;
            let changed = tx.execute(
                "UPDATE app_wallets SET balance_credit_micros=balance_credit_micros-?2,updated_at=?3
                 WHERE id=?1 AND status='active'
                   AND balance_credit_micros-?2>=frozen_credit_micros",
                params![wallet_id,charge,now],
            )?;
            if changed != 1 {
                return Err(rusqlite::Error::InvalidParameterName(
                    "wallet_insufficient_balance".to_string(),
                ));
            }
            let balance_after: i64 = tx.query_row(
                "SELECT balance_credit_micros FROM app_wallets WHERE id=?1",
                [wallet_id],
                |row| row.get(0),
            )?;
            tx.execute(
                "INSERT INTO app_wallet_ledger_entries(id,wallet_id,entry_kind,
                   amount_credit_micros,balance_after_credit_micros,request_log_id,api_key_id,
                   pricing_rule_id,raw_usage_json,note,created_by_user_id,created_at)
                 VALUES(?1,?2,'request_charge',?3,?4,?5,?6,NULL,?7,'model_catalog_v2',NULL,?8)",
                params![
                    format!("wl_request_{}", input.request_log_id),
                    wallet_id,
                    -charge,
                    balance_after,
                    input.request_log_id,
                    input.api_key_id,
                    input.raw_usage_json,
                    now
                ],
            )?;
        }
        tx.commit()?;
        self.get_charge_snapshot_v2(input.request_log_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_formula_charges_cached_subset_once() {
        let tier = ModelPriceTierV2 {
            min_input_tokens: 0,
            input_microusd_per_1m: 2_000_000,
            cached_input_microusd_per_1m: 200_000,
            output_microusd_per_1m: 10_000_000,
        };
        let result = compute_charge_v2(100, 40, 10, &tier, 1_500).unwrap();
        assert_eq!(result.uncached_input_tokens, 60);
        assert_eq!(result.numerator, 228_000_000);
        assert_eq!(result.base_cost_microusd, 228);
        assert_eq!(result.charged_cost_microusd, 342);
    }

    #[test]
    fn cached_above_input_is_clamped_and_tier_boundary_is_exact() {
        let storage = Storage::open_in_memory().unwrap();
        storage.init().unwrap();
        let (_, low) = storage
            .select_model_price_tier_v2("gpt-5.4", 271_999)
            .unwrap()
            .unwrap();
        let (_, high) = storage
            .select_model_price_tier_v2("gpt-5.4", 272_000)
            .unwrap()
            .unwrap();
        assert_eq!(low.min_input_tokens, 0);
        assert_eq!(high.min_input_tokens, 272_000);
        let result = compute_charge_v2(10, 20, 0, &low, 1_000).unwrap();
        assert_eq!(result.uncached_input_tokens, 0);
        assert_eq!(
            result.numerator,
            10_i128 * i128::from(low.cached_input_microusd_per_1m)
        );
    }

    #[test]
    fn missing_price_is_rejected_and_snapshot_is_idempotent() {
        let storage = Storage::open_in_memory().unwrap();
        storage.init().unwrap();
        storage.conn.execute("INSERT INTO request_logs(request_path,method,created_at) VALUES('/v1/responses','POST',1)",[]).unwrap();
        let request_log_id = storage.conn.last_insert_rowid();
        let mut input = ChargeSnapshotInputV2 {
            request_log_id,
            model_slug: "gpt-5.6-sol".into(),
            usage_source: "actual".into(),
            input_tokens: 1,
            cached_input_tokens: 0,
            output_tokens: 1,
            rate_multiplier_millis: 1_000,
            ..Default::default()
        };
        assert!(storage
            .record_charge_snapshot_v2(&input)
            .unwrap_err()
            .to_string()
            .contains("model_price_missing"));
        input.model_slug = "gpt-5.4-mini".into();
        let first = storage.record_charge_snapshot_v2(&input).unwrap();
        let second = storage.record_charge_snapshot_v2(&input).unwrap();
        assert_eq!(first, second);
    }
}
