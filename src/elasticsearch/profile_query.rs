use crate::elasticsearch::{Elasticsearch, ElasticsearchError};
use crate::zdbquery::ZDBQuery;
use pgx::*;
use serde_json::*;

pub struct ElasticsearchProfileQueryRequest {
    elasticsearch: Elasticsearch,
    query: ZDBQuery,
}

impl ElasticsearchProfileQueryRequest {
    pub fn new(elasticsearch: &Elasticsearch, query: ZDBQuery) -> Self {
        ElasticsearchProfileQueryRequest {
            elasticsearch: elasticsearch.clone(),
            query,
        }
    }

    pub fn execute(&self) -> std::result::Result<serde_json::Value, ElasticsearchError> {
        let body = json! {
            {
                "profile": true,
                "query": self.query.query_dsl().expect("no query dsl")
            }
        };

        let mut url = String::new();
        url.push_str(&self.elasticsearch.base_url());
        url.push_str("/_search");
        url.push_str("?size=0");
        url.push_str("&filter_path=profile");
        Elasticsearch::execute_request(
            reqwest::Client::new()
                .post(&url)
                .header("content-type", "application/json")
                .body(serde_json::to_string(&body).expect("failed to generate body")),
            |status, body| match serde_json::from_str::<serde_json::Value>(&body) {
                Ok(value) => Ok(value),
                Err(e) => Err(ElasticsearchError(Some(status), e.to_string())),
            },
        )
    }
}

#[pg_extern(immutable, parallel_safe)]
fn profile_query(index: PgRelation, query: ZDBQuery) -> JsonB {
    JsonB(
        Elasticsearch::new(&index)
            .profile_query(query)
            .execute()
            .expect("failed to execute profile query request"),
    )
}