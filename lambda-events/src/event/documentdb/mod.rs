pub mod events;

use self::events::{
    delete_event::ChangeDeleteEvent, drop_database_event::ChangeDropDatabaseEvent, drop_event::ChangeDropEvent,
    insert_event::ChangeInsertEvent, invalidate_event::ChangeInvalidateEvent, rename_event::ChangeRenameEvent,
    replace_event::ChangeReplaceEvent, update_event::ChangeUpdateEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "operationType", rename_all = "camelCase")]
pub enum ChangeEvent {
    Insert(ChangeInsertEvent),
    Delete(ChangeDeleteEvent),
    Drop(ChangeDropEvent),
    DropDatabase(ChangeDropDatabaseEvent),
    Invalidate(ChangeInvalidateEvent),
    Replace(ChangeReplaceEvent),
    Update(ChangeUpdateEvent),
    Rename(ChangeRenameEvent),
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

    fn test_example(data: &[u8]) {
        let mut data = data.to_vec();
        let parsed: Event = aws_lambda_json_impl::from_slice(data.as_mut_slice()).unwrap();
        let mut output = aws_lambda_json_impl::to_string(&parsed).unwrap().into_bytes();
        let reparsed: Event = aws_lambda_json_impl::from_slice(output.as_mut_slice()).unwrap();

        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn example_documentdb_insert_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-insert-event.json"));
    }

    #[test]
    fn example_documentdb_delete_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-delete-event.json"));
    }

    #[test]
    fn example_documentdb_drop_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-drop-event.json"));
    }

    #[test]
    fn example_documentdb_replace_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-replace-event.json"));
    }

    #[test]
    fn example_documentdb_update_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-update-event.json"));
    }

    #[test]
    fn example_documentdb_rename_event() {
        test_example(include_bytes!("../../fixtures/example-documentdb-rename-event.json"));
    }

    #[test]
    fn example_documentdb_invalidate_event() {
        test_example(include_bytes!(
            "../../fixtures/example-documentdb-invalidate-event.json"
        ));
    }

    #[test]
    fn example_documentdb_drop_database_event() {
        test_example(include_bytes!(
            "../../fixtures/example-documentdb-drop-database-event.json"
        ));
    }
}
