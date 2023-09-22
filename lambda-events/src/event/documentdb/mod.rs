pub mod events;

use std::collections::HashMap;

use self::events::insert_event::ChangeInsertEvent;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentDbInnerEvent {
    pub event: ChangeInsertEvent<HashMap<String, Value>>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDbEvent {
    #[serde(default)]
    pub event_source_arn: Option<String>,
    pub events: Vec<DocumentDbInnerEvent>,
    #[serde(default)]
    pub event_source: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "documentdb")]
    fn example_documentdb_insert_event() {
        let data = include_bytes!("../../fixtures/example-documentdb-insert-event.json");

        let parsed: DocumentDbEvent = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: DocumentDbEvent = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}
