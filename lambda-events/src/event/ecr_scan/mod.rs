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
    #[serde(default)]
    #[serde(rename = "CRITICAL")]
    pub critical: Option<i64>,
    #[serde(default)]
    #[serde(rename = "HIGH")]
    pub high: Option<i64>,
    #[serde(default)]
    #[serde(rename = "MEDIUM")]
    pub medium: Option<i64>,
    #[serde(default)]
    #[serde(rename = "LOW")]
    pub low: Option<i64>,
    #[serde(default)]
    #[serde(rename = "INFORMATIONAL")]
    pub informational: Option<i64>,
    #[serde(default)]
    #[serde(rename = "UNDEFINED")]
    pub undefined: Option<i64>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "ecr_scan")]
    fn example_ecr_image_scan_event() {
        let data = include_bytes!("../../fixtures/example-ecr-image-scan-event.json");
        let parsed: EcrScanEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EcrScanEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "ecr_scan")]
    fn example_ecr_image_scan_event_with_missing_severities() {
        let data = include_bytes!("../../fixtures/example-ecr-image-scan-event-with-missing-severities.json");
        let parsed: EcrScanEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: EcrScanEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
