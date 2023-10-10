pub mod events;

use self::events::{
    delete_event::ChangeDeleteEvent, drop_event::ChangeDropEvent, insert_event::ChangeInsertEvent,
    replace_event::ChangeReplaceEvent, update_event::ChangeUpdateEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "operationType", rename_all = "camelCase")]
pub enum ChangeEvent {
    Insert(ChangeInsertEvent),
    Delete(ChangeDeleteEvent),
    Drop(ChangeDropEvent),
    Replace(ChangeReplaceEvent),
    Update(ChangeUpdateEvent),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentDbInnerEvent {
    pub event: ChangeEvent,
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
#[cfg(feature = "documentdb")]
mod test {
    use super::*;

    pub type Event = DocumentDbEvent;

    #[test]
    fn example_documentdb_insert_event() {
        let data = include_bytes!("../../fixtures/example-documentdb-insert-event.json");

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_documentdb_delete_event() {
        let data = include_bytes!("../../fixtures/example-documentdb-delete-event.json");

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_documentdb_drop_event() {
        let data = include_bytes!("../../fixtures/example-documentdb-drop-event.json");

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_documentdb_replace_event() {
        let data = include_bytes!("../../fixtures/example-documentdb-replace-event.json");

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_documentdb_update_event() {
        let data = include_bytes!("../../fixtures/example-documentdb-update-event.json");

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();

        assert_eq!(parsed, reparsed);
    }
}
