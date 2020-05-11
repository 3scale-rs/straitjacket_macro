// There is currently a requirement to test a macro crate outside its own crate.
use straitjacket_macro::straitjacket;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct Link {
    rel: String,
    href: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
struct MyMetadata {
    created_at: String,
    updated_at: String,
    links: Vec<Link>,
}

#[straitjacket(metadata = "MyMetadata")]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize, Default)]
pub struct MappingRule {
    id: u64,
    metric_id: u64,
    pattern: String,
    http_method: String,
    delta: u64,
    position: u64,
    last: bool,
}

#[test]
// This is just a compile test, as we should generate the below types
fn t() {
    let mr = MappingRule { ..Default::default() };
    let _ = MappingRules { mapping_rules: vec![] };
    let _ = MappingRuleTag::Tag(MappingRuleAndMetadata { item: mr, metadata: None });
    assert!(true);
}

#[test]
fn it_serializes() {
    let mapping_rules = MappingRules::from(vec![
        MappingRule {
            id: 375841,
            metric_id: 2555418191879,
            pattern: "/".into(),
            http_method: "GET".into(),
            delta: 1,
            position: 1,
            last: false,
        },
        MappingRule {
            id: 375842,
            metric_id: 2555418191880,
            pattern: "/".into(),
            http_method: "POST".into(),
            delta: 1,
            position: 2,
            last: false,
        }
    ]);
    println!("{}", serde_json::to_string_pretty(&mapping_rules).unwrap());
}

#[test]
fn it_parses() {
    let body = r##"{
        "mapping_rules": [
          {
            "mapping_rule": {
              "id": 375841,
              "metric_id": 2555418191879,
              "pattern": "/",
              "http_method": "GET",
              "delta": 1,
              "position": 1,
              "last": false,
              "created_at": "2019-03-19T09:04:35Z",
              "updated_at": "2019-03-19T09:04:39Z",
              "links": [
                {
                  "rel": "self",
                  "href": "/admin/api/services/2555417777820/proxy/mapping_rules/375841"
                },
                {
                  "rel": "service",
                  "href": "/admin/api/services/2555417777820"
                },
                {
                  "rel": "proxy",
                  "href": "/admin/api/services/2555417777820/proxy"
                }
              ]
            }
          },
          {
            "mapping_rule": {
              "id": 375842,
              "metric_id": 2555418191880,
              "pattern": "/",
              "http_method": "POST",
              "delta": 1,
              "position": 2,
              "last": false,
              "created_at": "2019-03-19T09:04:36Z",
              "updated_at": "2019-03-19T09:04:39Z",
              "links": [
                {
                  "rel": "self",
                  "href": "/admin/api/services/2555417777820/proxy/mapping_rules/375842"
                },
                {
                  "rel": "service",
                  "href": "/admin/api/services/2555417777820"
                },
                {
                  "rel": "proxy",
                  "href": "/admin/api/services/2555417777820/proxy"
                }
              ]
            }
          }
        ]}"##;
    let mapping_rules: Result<MappingRules, _> = serde_json::from_str(&body);
    assert!(mapping_rules.is_ok());
    let mapping_rules = mapping_rules.unwrap();
    println!("PARSED:\n{}", serde_json::to_string_pretty(&mapping_rules).unwrap());
}
