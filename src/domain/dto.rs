use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    #[serde(rename = "trapAccountName")]
    pub trap_account_name: String,
    #[serde(rename = "atcoderAccountName")]
    pub atcoder_account_name: Option<String>,
    #[serde(rename = "atcoderRating")]
    pub atcoder_rating: Option<i32>,
    #[serde(rename = "heuristicRating")]
    pub heuristic_rating: Option<i32>,
    #[serde(rename = "isAlgoTeam")]
    pub is_algo_team: Option<bool>,
}
