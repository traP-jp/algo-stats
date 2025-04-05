#![allow(non_snake_case, unused)]
use anyhow::Result;
use flate2::read::GzDecoder;
use async_trait::async_trait;
use std::io::Read;
use std::collections::HashMap;

static ATCODER_WAIT_TIME_MS: u64 = 1000;

pub struct DetailUpdaterImpl {
    http_client: reqwest::Client,
}

/*
  {
    "IsRated": true,
    "Place": 1673,
    "OldRating": 0,
    "NewRating": 60,
    "Performance": 838,
    "InnerPerformance": 838,
    "ContestScreenName": "arc154.contest.atcoder.jp",
    "ContestName": "AtCoder Regular Contest 154",
    "ContestNameEn": "",
    "EndTime": "2023-01-22T23:00:00+09:00"
  },
*/
#[derive(Debug, Clone, serde::Deserialize)]
struct ContestResultDto {
    IsRated: bool,
    Place: i32,
    OldRating: i32,
    NewRating: i32,
    Performance: i32,
    InnerPerformance: i32,
    ContestScreenName: String,
    ContestName: String,
    ContestNameEn: String,
    EndTime: String,
}

#[async_trait]
impl crate::domain::detail_updater::DetailedInfoUpdater for DetailUpdaterImpl {
    async fn get(
        &self,
        usernames: Vec<String>,
    ) -> Result<HashMap<String, crate::domain::entity::AcDetailedInfo>> {
        tracing::info!("Starting to fetch from atcoder");
        let mut results = HashMap::new();
        for username in usernames {
            let data = self.get_inner(&username, false).await?;
            let heur_data = self.get_inner(&username, true).await?;
            let contest_results: Vec<crate::domain::entity::ContestResult> = data
                .into_iter()
                .map(|dto| crate::domain::entity::ContestResult {
                    is_rated: dto.IsRated,
                    place: dto.Place,
                    old_rating: dto.OldRating,
                    new_rating: dto.NewRating,
                    diff: dto.NewRating - dto.OldRating,
                    performance: dto.Performance,
                    contest_screen_name: dto.ContestScreenName,
                    contest_name: dto.ContestName,
                    end_time: dto.EndTime,
                })
                .collect();
            let heur_results: Vec<crate::domain::entity::ContestResult> = heur_data
                .into_iter()
                .map(|dto| crate::domain::entity::ContestResult {
                    is_rated: dto.IsRated,
                    place: dto.Place,
                    old_rating: dto.OldRating,
                    new_rating: dto.NewRating,
                    diff: dto.NewRating - dto.OldRating,
                    performance: dto.Performance,
                    contest_screen_name: dto.ContestScreenName,
                    contest_name: dto.ContestName,
                    end_time: dto.EndTime,
                })
                .collect();
            results.insert(
                username.clone(),
                crate::domain::entity::AcDetailedInfo {
                    algo_rating: contest_results,
                    heur_rating: heur_results,
                },
            );
            // Sleep to avoid hitting the rate limit
            tokio::time::sleep(std::time::Duration::from_millis(ATCODER_WAIT_TIME_MS)).await;
        }
        Ok(results)
    }
}

impl DetailUpdaterImpl {
    pub fn new() -> Self {
        let http_client = reqwest::Client::new();
        DetailUpdaterImpl { http_client }
    }

    async fn get_inner(
        &self,
        username: &str,
        is_heur: bool,
    ) -> Result<Vec<ContestResultDto>> {
        let query_param = if is_heur {
            "?contestType=heuristic"
        } else {
            ""
        };
        let url = format!("https://atcoder.jp/users/{}/history/json{}", username, query_param);
        tracing::info!("Fetching from {}", url);
        let response = self.http_client
            .get(&url)
            .header("Accept-Encoding", "gzip")
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request: {}", e))?;
        match response.status() {
            reqwest::StatusCode::OK => {
                if Some("gzip") != response.headers().get("Content-Encoding").map(|v| v.to_str().unwrap()) {
                    tracing::warn!("Response is not gzipped");
                    return Ok(vec![]);
                }
                let bytes = response.bytes().await?;
                let mut gz = GzDecoder::new(bytes.as_ref());
                let mut s = String::new();
                gz.read_to_string(&mut s)
                    .map_err(|e| anyhow::anyhow!("Failed to decompress response: {}", e))?;
                let data: Vec<ContestResultDto> = serde_json::from_str(&s)
                    .map_err(|e| anyhow::anyhow!("Failed to parse JSON: {}", e))?;
                Ok(data)
            }
            other => {
                Err(anyhow::anyhow!("Failed to fetch data: {}", other))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio;
    use crate::domain::detail_updater::DetailedInfoUpdater as _;
    #[tokio::test]
    async fn test_get() {
        let updater = DetailUpdaterImpl::new();
        let usernames = vec!["Dye8128".to_string(), "chokudai".to_string()];
        let result = updater
            .get(usernames.clone())
            .await;
        match result {
            Ok(data) => {
                assert_eq!(data.len(), usernames.len());
                for (username, info) in data.iter() {
                    println!("Username: {}", username);
                    for contest in &info.algo_rating {
                        println!("Contest: {}, Place: {}", contest.contest_name, contest.place);
                    }
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
}
