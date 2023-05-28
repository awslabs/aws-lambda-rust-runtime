use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcrScanEvent {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    #[serde(rename = "detail-type")]
    pub detail_type: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub time: Option<String>,
    #[serde(default)]
    pub region: Option<String>,
    pub resources: Vec<String>,
    #[serde(default)]
    pub account: Option<String>,
    pub detail: EcrScanEventDetailType,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcrScanEventDetailType {
    #[serde(default)]
    #[serde(rename = "scan-status")]
    pub scan_status: Option<String>,
    #[serde(default)]
    #[serde(rename = "repository-name")]
    pub repository_name: Option<String>,
    #[serde(rename = "finding-severity-counts")]
    pub finding_severity_counts: EcrScanEventFindingSeverityCounts,
    #[serde(default)]
    #[serde(rename = "image-digest")]
    pub image_digest: Option<String>,
    #[serde(rename = "image-tags")]
    pub image_tags: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcrScanEventFindingSeverityCounts {
    #[serde(rename = "CRITICAL")]
    pub critical: i64,
    #[serde(rename = "HIGH")]
    pub high: i64,
    #[serde(rename = "MEDIUM")]
    pub medium: i64,
    #[serde(rename = "LOW")]
    pub low: i64,
    #[serde(rename = "INFORMATIONAL")]
    pub informational: i64,
    #[serde(rename = "UNDEFINED")]
    pub undefined: i64,
}

#[cfg(test)]
mod test {
    use super::*;

    use serde_json;

    #[test]
    #[cfg(feature = "ecr_scan")]
    fn example_ecr_image_scan_event() {
        let data = include_bytes!("../../fixtures/example-ecr-image-scan-event.json");
        let parsed: EcrScanEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EcrScanEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
