pub mod events;

use self::events::{insert_event::ChangeInsertEvent, 
                   delete_event::ChangeDeleteEvent,
                   drop_event::ChangeDropEvent, 
                   drop_database_event::ChangeDropDatabaseEvent};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ChangeEvent<T: Serialize> {
    ChangeInsertEvent(ChangeInsertEvent<T>),
    ChangeDeleteEvent(ChangeDeleteEvent),
    ChangeDropEvent(ChangeDropEvent),
    ChangeDropDatabaseEvent(ChangeDropDatabaseEvent),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentDbInnerEvent<T: Serialize> {
    pub event: ChangeEvent<T>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentDbEvent<T: Serialize> {
    #[serde(default)]
    pub event_source_arn: Option<String>,
    pub events: Vec<DocumentDbInnerEvent<T>>,
    #[serde(default)]
    pub event_source: Option<String>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[cfg(feature = "documentdb")]
    fn example_documentdb_insert_event() {
        use std::collections::HashMap;

        use serde_json::Value;

        let data = include_bytes!("../../fixtures/example-documentdb-insert-event.json");

        type Event = DocumentDbEvent<HashMap<String, Value>>;

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }


    #[test]
    #[cfg(feature = "documentdb")]
    fn example_documentdb_delete_event() {
        use std::collections::HashMap;

        use serde_json::Value;

        let data = include_bytes!("../../fixtures/example-documentdb-delete-event.json");

        type Event = DocumentDbEvent<HashMap<String, Value>>;

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "documentdb")]
    fn example_documentdb_drop_event(){
        use std::collections::HashMap;

        use serde_json::Value;

        let data = include_bytes!("../../fixtures/example-documentdb-drop-event.json");

        type Event = DocumentDbEvent<HashMap<String, Value>>;

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    #[cfg(feature = "documentdb")]
    fn example_documentdb_drop_database_event(){
        use std::collections::HashMap;

        use serde_json::Value;

        let data = include_bytes!("../../fixtures/example-documentdb-drop-database-event.json");

        type Event = DocumentDbEvent<HashMap<String, Value>>;

        let parsed: Event = serde_json::from_slice(data).unwrap();
        let output: String = serde_json::to_string(&parsed).unwrap();
        let reparsed: Event = serde_json::from_slice(output.as_bytes()).unwrap();
        assert_eq!(parsed, reparsed);
    }
}