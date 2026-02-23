use anyhow::{Result, bail};

use crate::storage::models::{
    BondingPostionDbModel,
    HolderDbModel, HolderStats, PremarketInfoDbModel, CommunityInfoDbModel, CommunityLinkDbModel
};
use crate::models::premarket::{
    PremarketInfoServiceModel,
    BondingPostion
};
use sqlx::{PgPool};
use uuid::Uuid;
use chrono::Utc;

pub async fn get_user_concept(
    pool: &PgPool,
    creator_id: &Uuid
) -> Result<Option<PremarketInfoServiceModel>> {
    let premarket_db = sqlx::query_as::<_, PremarketInfoDbModel>(
        r#"
        SELECT * FROM premarket_info WHERE creator_id = $1 AND state = 'concept' LIMIT 1;
        "#
    )
    .bind(creator_id)
    .fetch_optional(pool)
    .await?;

    if let Some(pm_db) = premarket_db {
        let premarket_service: PremarketInfoServiceModel = pm_db.try_into()?;
        Ok(Some(premarket_service))
    } else {
        Ok(None)
    }
}


pub async fn get_premarket_id_by_bc_address(
    pool: &PgPool,
    bc_address: &str,
) -> Result<Option<Uuid>> {
    let id = sqlx::query_scalar::<_, Uuid>(r#"SELECT id FROM premarket_info WHERE bc_address = $1 LIMIT 1;"#)
    .bind(bc_address)
    .fetch_optional(pool)
    .await?;

    Ok(id)
}
pub async fn get_premarket_info_by_name(
    pool: &PgPool,
    premarket_name: &str,
) -> Result<Option<(PremarketInfoDbModel, CommunityInfoDbModel, Vec<CommunityLinkDbModel>)>> {
    let premarket = sqlx::query_as::<_, PremarketInfoDbModel>(
        r#"
        SELECT *
        FROM premarket_info
        WHERE short_url_name = $1
        LIMIT 1
        "#
    )
    .bind(premarket_name)
    .fetch_optional(pool)
    .await?;

    if let Some(pm) = &premarket {
        let community = sqlx::query_as::<_, CommunityInfoDbModel>(
            r#"SELECT * FROM community_info WHERE id = $1"#
        )
        .bind(pm.id)
        .fetch_optional(pool)
        .await?;

        let (community, links_db) = if let Some(cm) = community {
            let links = sqlx::query_as::<_, CommunityLinkDbModel>(
                r#"SELECT * FROM community_links WHERE community_info_id = $1"#
            )
            .bind(cm.id)
            .fetch_all(pool)
            .await?;

            (cm, links)
        } else {
            (
                CommunityInfoDbModel {
                    id: pm.id,
                    description: "".to_string(),
                    token_banner_url: None,
                },
                vec![],
            )
        };

        Ok(Some((pm.clone(), community, links_db)))
    } else {
        Ok(None)
    }
}


// todo: fix me to return service model with convertor simular to PremarketInfoServiceModel
pub async fn get_premarket_info_by_bc_address(
    pool: &PgPool,
    bc_address: &str
) -> Result<Option<(PremarketInfoDbModel, CommunityInfoDbModel, Vec<CommunityLinkDbModel>)>> {
    let premarket = sqlx::query_as::<_, PremarketInfoDbModel>(
        r#"
        SELECT * FROM premarket_info WHERE bc_address = $1
        "#
    )
    .bind(bc_address)
    .fetch_optional(pool)
    .await?;

    if let Some(pm) = &premarket {
        let community = sqlx::query_as::<_, CommunityInfoDbModel>(
    r#"
    SELECT * FROM community_info WHERE id = $1
    "#
)
            .bind(pm.id)
            .fetch_optional(pool)
            .await?;

            let (community, links_db) = if let Some(cm) = community {
                let links = sqlx::query_as::<_, CommunityLinkDbModel>(
                    r#"SELECT * FROM community_links WHERE community_info_id = $1"#
                )
                .bind(cm.id)
                .fetch_all(pool)
                .await?;

                (cm, links)
            } else {
                println!("community not found! set dummy");
                (
                    CommunityInfoDbModel {
                        id: pm.id,
                        description: "".to_string(),
                        token_banner_url: None,
                    },
                    vec![],
                )
            };


        Ok(Some((pm.clone(), community, links_db)))
    } else {
        Ok(None)
    }
}


// todo: fix me to return service model with convertor simular to PremarketInfoServiceModel
pub async fn get_premarket_info_by_id(
    pool: &PgPool,
    premarket_id: &Uuid
) -> Result<Option<(PremarketInfoDbModel, CommunityInfoDbModel, Vec<CommunityLinkDbModel>)>> {
    let premarket = sqlx::query_as::<_, PremarketInfoDbModel>(
        r#"
        SELECT * FROM premarket_info WHERE id = $1
        "#
    )
    .bind(premarket_id)
    .fetch_optional(pool)
    .await?;

    if let Some(pm) = &premarket {
        let community = sqlx::query_as::<_, CommunityInfoDbModel>(
    r#"
    SELECT * FROM community_info WHERE id = $1
    "#
)
            .bind(pm.id)
            .fetch_optional(pool)
            .await?;

            let (community, links_db) = if let Some(cm) = community {
                let links = sqlx::query_as::<_, CommunityLinkDbModel>(
                    r#"SELECT * FROM community_links WHERE community_info_id = $1"#
                )
                .bind(cm.id)
                .fetch_all(pool)
                .await?;

                (cm, links)
            } else {
                println!("community not found! set dummy");
                (
                    CommunityInfoDbModel {
                        id: pm.id,
                        description: "".to_string(),
                        token_banner_url: None,
                    },
                    vec![],
                )
            };


        Ok(Some((pm.clone(), community, links_db)))
    } else {
        Ok(None)
    }
}


// right method -> because service works with service model. and shouldn't know about DB model
pub async fn get_main_premarket_info_by_bc_address(
    pool: &PgPool,
    bc_address: &str,
) -> Result<Option<PremarketInfoServiceModel>> {
    let pm_db = sqlx::query_as::<_, PremarketInfoDbModel>(
        r#"SELECT * FROM premarket_info WHERE bc_address = $1"#
    )
    .bind(bc_address)
    .fetch_optional(pool)
    .await?;

    let pm_db = match pm_db {
        Some(v) => v,
        None => return Ok(None),
    };

    let premarket_info: PremarketInfoServiceModel = pm_db.try_into()?;

    Ok(Some(premarket_info))
}
 

pub async fn get_list(
    pool: &PgPool,
    user_id_opt: Option<Uuid>,
    cursor: i64,
    limit: i64,
) -> Result<Option<(Vec<PremarketInfoDbModel>, i64)>> {
    let limit = limit.clamp(1, 200);
    let offset = cursor.max(0);

    let total: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM premarket_info
        WHERE
          state != 'concept' 
          AND (
            is_hided = FALSE
            OR (
                $1::uuid IS NOT NULL
                AND creator_id = $1::uuid
            )
          )
        "#,
    )
    .bind(user_id_opt)
    .fetch_one(pool)
    .await?;

    let rows: Vec<PremarketInfoDbModel> = sqlx::query_as::<_, PremarketInfoDbModel>(
        r#"
        SELECT *
        FROM premarket_info
        WHERE
          state != 'concept' 
          AND (
            is_hided = FALSE
            OR (
                $1::uuid IS NOT NULL
                AND creator_id = $1::uuid
            )
          )
        ORDER BY premarket_created DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id_opt)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(Some((rows, total)))
}


pub async fn create_premarket_and_community(
    pool: &PgPool,
    premarket: &PremarketInfoDbModel,
    community: &CommunityInfoDbModel,
    links: Option<Vec<CommunityLinkDbModel>>,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    let created_pm: PremarketInfoDbModel = sqlx::query_as::<_, PremarketInfoDbModel>(
        r#"
        INSERT INTO premarket_info (
            id,
            creator_id,
            creator_address,
            bc_address,
            data_uri,
            name,
            description,
            symbol,
            image_url,
            telegram,
            twitter,
            web_site,
            premarket_goal_sol_lamp,
            premarket_deadline,
            premarket_created,
            state,
            mint_address,
            is_hided,
            short_url_name
        )
        VALUES (
            $1, $2, $3, $4, $5,
            $6, $7, $8, $9, $10,
            $11, $12, $13, $14, $15,
            $16, $17, $18, $19
        )
        RETURNING *
        "#
    )
    .bind(premarket.id)
    .bind(premarket.creator_id)
    .bind(&premarket.creator_address)
    .bind(&premarket.bc_address)
    .bind(&premarket.data_uri)
    .bind(&premarket.name)
    .bind(&premarket.description)
    .bind(&premarket.symbol)
    .bind(&premarket.image_url)
    .bind(&premarket.telegram)
    .bind(&premarket.twitter)
    .bind(&premarket.web_site)
    .bind(premarket.premarket_goal_sol_lamp)
    .bind(premarket.premarket_deadline)
    .bind(premarket.premarket_created)
    .bind(&premarket.state)
    .bind(&premarket.mint_address)
    .bind(&premarket.is_hided)
    .bind(&premarket.short_url_name)
    .fetch_one(&mut tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO community_info (id, description, token_banner_url)
        VALUES ($1, $2, $3)
        "#
    )
    .bind(created_pm.id)
    .bind(&community.description)
    .bind(&community.token_banner_url)
    .execute(&mut tx)
    .await?;

    
    if let Some(link_list) = links {
        for link in link_list {
            sqlx::query(
                r#"
                INSERT INTO community_links (id, community_info_id, text, url, type)
                VALUES ($1, $2, $3, $4, $5)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(created_pm.id)
            .bind(&link.text)
            .bind(&link.url)
            .bind(&link.r#type.to_string())
            .execute(&mut tx)
            .await?;
        }
    }


    if let Err(e) = tx.commit().await {
        eprintln!("❌ Failed to commit transaction: {:?}", e);
        return Err(e.into());
    }
    Ok(())
}

pub async fn hard_delete_premarket_by_id(pool: &PgPool, premarket_id: Uuid) -> Result<u64> {
    let res = sqlx::query(r#"DELETE FROM premarket_info WHERE id = $1"#)
        .bind(premarket_id)
        .execute(pool)
        .await?;

    let affected = res.rows_affected();
    if affected == 0 {
        bail!("premarket not found");
    }

    Ok(affected)
}


pub async fn update_availability_info(
    pool: &PgPool,
    bc_address: &str,
    is_hided: Option<bool>,
    is_whitelist_enabled: Option<bool>,
    short_url_name: Option<String>,
) -> Result<()> {
    let id_option = get_premarket_id_by_bc_address(pool, bc_address).await?;
    let premarket_info_id = id_option.ok_or(sqlx::Error::RowNotFound)?;

    let _result = sqlx::query(
        r#"
        UPDATE premarket_info
            SET
                is_hided = COALESCE($2, is_hided),
                short_url_name = COALESCE($3, short_url_name),
                is_whitelist_enabled = COALESCE($4, is_whitelist_enabled)
            WHERE id = $1
        "#,
    )
    .bind(premarket_info_id)
    .bind(is_hided)
    .bind(short_url_name)
    .bind(is_whitelist_enabled)
    .execute(pool)
    .await?;

    return Ok(());
}

pub async fn update_community_info(
    pool: &PgPool,
    bc_address: &str,
    description: &str,
    token_banner_url: Option<&str>,
    links: Option<Vec<CommunityLinkDbModel>>,
) -> Result<()> {
    let mut tx = pool.begin().await?;

    let premarket_id: Uuid = sqlx::query_scalar(
        r#"SELECT id FROM premarket_info WHERE bc_address = $1"#,
    )
    .bind(bc_address)
    .fetch_one(&mut tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE community_info
        SET description = $1, token_banner_url = $2
        WHERE id = $3
        "#
    )
    .bind(description)
    .bind(token_banner_url)                    // <-- теперь Option<&str> => NULL при None
    .bind(premarket_id)
    .execute(&mut tx)
    .await?;

    sqlx::query(r#"DELETE FROM community_links WHERE community_info_id = $1"#)
        .bind(premarket_id)
        .execute(&mut tx)
        .await?;

    if let Some(link_list) = links {
        for link in link_list {
            sqlx::query(
                r#"
                INSERT INTO community_links (id, community_info_id, text, url, type)
                VALUES ($1, $2, $3, $4, $5)
                "#
            )
            .bind(Uuid::new_v4())
            .bind(premarket_id)
            .bind(&link.text)
            .bind(&link.url)
            .bind(&link.r#type.to_string())
            .execute(&mut tx)
            .await?;
        }
    }

    tx.commit().await?;
    Ok(())
}

pub async fn insert_holder(
    pool: &PgPool,
    premarket_pubkey: &str,
    holder: &HolderDbModel,
) -> Result<()> {
    let id_option = get_premarket_id_by_bc_address(pool, premarket_pubkey).await?;
    if let Some(id) = &id_option {
        sqlx::query(
            r#"
            INSERT INTO premarket_holders (
                id,
                premarket_info_id,
                holder_id,
                holder_wallet,
                amount_lamport,
                join_timestamp,
                out_timestamp,
                claimed,
                amount_token,
                claimed_amount_token,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, now())
            "#,
        )
        .bind(holder.id)
        .bind(id)
        .bind(holder.holder_id)
        .bind(&holder.holder_wallet)
        .bind(holder.amount_lamport)
        .bind(holder.join_timestamp)
        .bind(holder.out_timestamp)
        .bind(holder.claimed)
        .bind(&holder.amount_token)
        .bind(&holder.claimed_amount_token)
        .execute(pool)
        .await?;

        return Ok(());
    }

    bail!("premarket not found");
}

pub async fn soft_delete_holder(
    pool: &PgPool,
    premarket_pubkey: &str,
    holder_wallet: &str,
    out_timestamp: i64,
) -> Result<u64> {
    let id_option = get_premarket_id_by_bc_address(pool, premarket_pubkey).await?;
    if let Some(premarket_info_id) = id_option {
        println!("out_timestamp:{}, premarket_info_id:{:?}, holder_wallet:{}", out_timestamp, premarket_info_id, holder_wallet);
        let result = sqlx::query(
            r#"
            UPDATE premarket_holders
            SET out_timestamp = $1
            WHERE premarket_info_id = $2 AND holder_wallet = $3 AND out_timestamp IS NULL
            "#,
        )
        .bind(out_timestamp)
        .bind(premarket_info_id)
        .bind(holder_wallet)
        .execute(pool)
        .await?;

        return Ok(result.rows_affected());
    }
    bail!("premarket not found");
}

pub async fn get_holders_by_premarket_address(
    pool: &PgPool,
    bc_address: &str,
    limit: i64,
) -> Result<HolderStats> {
    let premarket_info_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT id FROM premarket_info
        WHERE bc_address = $1
        "#,
    )
    .bind(bc_address)
    .fetch_one(pool)
    .await?;
    return get_holders_by_premarket_id(pool, premarket_info_id, limit).await;
}

pub async fn get_holders_by_premarket_id(
    pool: &PgPool,
    premarket_info_id: Uuid,
    limit: i64,
) -> Result<HolderStats> {
    let time_24h_ago = Utc::now().timestamp() - 24 * 60 * 60;

    let holders = sqlx::query_as::<_, HolderDbModel>(
        r#"
        SELECT 
            ph.id,
            ph.premarket_info_id,
            ph.holder_id,
            ph.holder_wallet,
            ph.amount_lamport,
            ph.join_timestamp,
            ph.out_timestamp,
            ph.claimed,
            u.avatar_url,
            u.username,
            ph.amount_token,
            ph.claimed_amount_token,
            ph.updated_at
        FROM premarket_holders ph
        LEFT JOIN users u ON u.id = ph.holder_id
        WHERE ph.premarket_info_id = $1
          AND ph.out_timestamp IS NULL
        ORDER BY ph.join_timestamp DESC
        LIMIT $2
        "#,
    )
    .bind(premarket_info_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;


    let total_active_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*) FROM premarket_holders
        WHERE premarket_info_id = $1 AND out_timestamp IS NULL
        "#,
    )
    .bind(premarket_info_id)
    .fetch_one(pool)
    .await?;

    let reserved_sol_lamp = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(amount_lamport)::BIGINT, 0)
        FROM premarket_holders
        WHERE premarket_info_id = $1 AND out_timestamp IS NULL
        "#,
    )
    .bind(premarket_info_id)
    .fetch_one(pool)
    .await?;

    let reserved_sol_24h_before_lamp = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(amount_lamport)::BIGINT, 0)
        FROM premarket_holders
        WHERE premarket_info_id = $1
          AND out_timestamp IS NULL
          AND join_timestamp <= $2
        "#,
    )
    .bind(premarket_info_id)
    .bind(time_24h_ago)
    .fetch_one(pool)
    .await?;

    let total_token_amount = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(amount_token)::BIGINT, 0)
        FROM premarket_holders
        WHERE premarket_info_id = $1
          AND out_timestamp IS NULL
        "#,
    )
    .bind(premarket_info_id)
    .fetch_one(pool)
    .await?;

    let total_claimed_token_amount = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(claimed_amount_token)::BIGINT, 0)
        FROM premarket_holders
        WHERE premarket_info_id = $1
          AND out_timestamp IS NULL
        "#,
    )
    .bind(premarket_info_id)
    .fetch_one(pool)
    .await?;

    Ok(HolderStats {
        holders,
        total_active_count,
        reserved_sol_lamp,
        reserved_sol_24h_before_lamp,
        total_token_amount,
        total_claimed_token_amount,
    })
}

pub async fn get_holder_entry_by_premarket_id(
    pool: &PgPool,
    premarket_info_id: Uuid,
    holder_wallet: &str,
) -> Result<Option<BondingPostion>> {
    let row: Option<BondingPostionDbModel> = sqlx::query_as::<_, BondingPostionDbModel>(
        r#"
        WITH holder AS (
            SELECT amount_lamport, join_timestamp, claimed, amount_token, claimed_amount_token
            FROM premarket_holders
            WHERE premarket_info_id = $1
              AND holder_wallet = $2
              AND out_timestamp IS NULL
            LIMIT 1
        )
        SELECT
            h.amount_lamport AS holder_amount,
            h.claimed AS is_claimed,
            (
                SELECT COALESCE(SUM(ph.amount_lamport), 0)::BIGINT
                FROM premarket_holders ph
                WHERE ph.premarket_info_id = $1
                  AND ph.out_timestamp IS NULL
                  AND ph.join_timestamp < h.join_timestamp
            ) AS total_amount,
            (
                SELECT COUNT(*)::BIGINT
                FROM premarket_holders ph
                WHERE ph.premarket_info_id = $1
                  AND ph.out_timestamp IS NULL
                  AND ph.join_timestamp < h.join_timestamp
            ) AS rank,
            h.amount_token AS amount_token,
            h.claimed_amount_token AS claimed_amount_token
        FROM holder h
        "#
    )
    .bind(premarket_info_id)
    .bind(holder_wallet)
    .fetch_optional(pool)
    .await?;

    let row = match row {
        Some(r) => r,
        None => return Ok(None),
    };

    let holder_amount = match row.holder_amount {
        Some(v) => v,
        None => return Ok(None),
    };

    Ok(Some(BondingPostion {
        amount_sol_lamp: holder_amount as u64,
        before_amount_sol_lamp: row.total_amount as u64,
        is_claimed: row.is_claimed,
        rank: row.rank as i64,
        amount_token: row.amount_token,
        claimed_amount_token: row.claimed_amount_token,
    }))
}


pub async fn update_premarket_state_to_finish(
    pool: &PgPool,
    premarket_pubkey: &str,
    new_state: &str,
    update_time: i64,
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        UPDATE premarket_info
        SET 
            state = $1,
            premarket_finished = $3
        WHERE bc_address = $2
        "#,
    )
    .bind(new_state)
    .bind(premarket_pubkey)
    .bind(update_time)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn update_premarket_state_to_start(
    pool: &PgPool,
    premarket_pubkey: &str,
    new_state: &str,
    update_time: i64,
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        UPDATE premarket_info
        SET 
            state = $1,
            premarket_created = $3
        WHERE bc_address = $2
        "#,
    )
    .bind(new_state)
    .bind(premarket_pubkey)
    .bind(update_time)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn get_lamports_before_timestamp(
    pool: &PgPool,
    premarket_pubkey: &str,
    timestamp: i64,
) -> Result<i64> {
    let premarket_info_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT id FROM premarket_info
        WHERE bc_address = $1
        "#,
    )
    .bind(premarket_pubkey)
    .fetch_one(pool)
    .await?;

    let lamports = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COALESCE(SUM(amount_lamport)::BIGINT, 0)
        FROM premarket_holders
        WHERE premarket_info_id = $1
          AND join_timestamp < $2
          AND (out_timestamp IS NULL OR out_timestamp > $2)
        "#,
    )
    .bind(premarket_info_id)
    .bind(timestamp)
    .fetch_one(pool)
    .await?;

    Ok(lamports)
}

pub async fn get_holder_join_timestamp(
    pool: &PgPool,
    premarket_pubkey: &str,
    holder_wallet: &str,
) -> Result<Option<i64>> {
    let premarket_info_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT id FROM premarket_info
        WHERE bc_address = $1
        "#,
    )
    .bind(premarket_pubkey)
    .fetch_one(pool)
    .await?;

    let timestamp = sqlx::query_scalar::<_, Option<i64>>(
        r#"
        SELECT join_timestamp
        FROM premarket_holders
        WHERE premarket_info_id = $1
          AND holder_wallet = $2
          AND out_timestamp IS NULL
        ORDER BY join_timestamp ASC
        LIMIT 1
        "#,
    )
    .bind(premarket_info_id)
    .bind(holder_wallet)
    .fetch_optional(pool)
    .await?;

    Ok(timestamp.flatten())
}

pub async fn update_premarket_deadline(
    pool: &PgPool,
    premarket_pubkey: &str,
    new_deadline: i64,
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        UPDATE premarket_info
        SET premarket_deadline = $1,
            is_extended = true
        WHERE bc_address = $2
        "#,
    )
    .bind(new_deadline)
    .bind(premarket_pubkey)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}


pub struct UpdateLinks {
    pub image_url: Option<String>,
    pub data_uri: String,
    pub telegram: Option<String>,
    pub twitter: Option<String>,
    pub web_site: Option<String>,
}
pub async fn update_all_links_premarket(
    pool: &PgPool,
    premarket_pubkey: &str,
    liks_to_update: UpdateLinks,
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        UPDATE premarket_info
        SET image_url = $1,
            data_uri = $2,
            telegram = $3,
            twitter = $4,
            web_site = $5,
        WHERE bc_address = $6
        "#,
    )
    .bind(liks_to_update.image_url)
    .bind(liks_to_update.data_uri)
    .bind(liks_to_update.telegram)
    .bind(liks_to_update.twitter)
    .bind(liks_to_update.web_site)
    .bind(premarket_pubkey)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}

pub async fn update_premarket_uri(
    pool: &PgPool,
    premarket_id: &Uuid,
    new_uri: &String,
    new_image_url: &String,
) -> Result<u64> {
    let res = sqlx::query(
        r#"
        UPDATE premarket_info
            SET 
                data_uri = $2, 
                image_url = $3
        WHERE id = $1
        "#,
    )
    .bind(premarket_id)
    .bind(new_uri)
    .bind(new_image_url)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}


pub async fn update_holder_claimed_status(
    pool: &PgPool,
    premarket_pubkey: &str,
    holder_wallet: &str,
    claimed: bool,
) -> Result<u64> {
    let premarket_info_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT id FROM premarket_info
        WHERE bc_address = $1
        "#,
    )
    .bind(premarket_pubkey)
    .fetch_one(pool)
    .await?;

    let res = sqlx::query(
        r#"
        UPDATE premarket_holders
        SET claimed = $1
        WHERE premarket_info_id = $2 
          AND holder_wallet = $3
          AND out_timestamp IS NULL
        "#,
    )
    .bind(claimed)
    .bind(premarket_info_id)
    .bind(holder_wallet)
    .execute(pool)
    .await?;

    Ok(res.rows_affected())
}
