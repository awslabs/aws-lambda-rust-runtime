use crate::Error;
use thiserror::Error;

/// A trait for receiving checkpoint/restore notifications.
///
/// The type that is interested in receiving a checkpoint/restore notification
/// implements this trait, and the instance created from that type is registered
/// inside the Runtime's list of resources, using the Runtime's register() method.
pub trait Resource {
    /// Invoked by Runtime as a notification about checkpoint (that snapshot is about to be taken)
    fn before_checkpoint(&self) -> Result<(), Error> {
        Ok(())
    }
    /// Invoked by Runtime as a notification about restore (snapshot was restored)
    fn after_restore(&self) -> Result<(), Error> {
        Ok(())
    }
}

/// Errors that can occur during checkpoint/restore hooks
#[derive(Error, Debug)]
pub enum CracError {
    /// Errors occurred during before_checkpoint() hook
    #[error("before checkpoint hooks errors: {0}")]
    BeforeCheckpointError(String),
    /// Errors occurred during after_restore() hook
    #[error("after restore hooks errors: {0}")]
    AfterRestoreError(String),
}

// implement a dummy Resource for unit type '()'
impl Resource for () {}

/// A context for CRAC resources.
pub struct Context<'a, T: Resource> {
    resources: Vec<&'a T>,
}

impl<'a, T: Resource> Default for Context<'a, T> {
    fn default() -> Self {
        Context::new()
    }
}

impl<'a, T: Resource> Context<'a, T> {
    /// Creates a new Context.
    pub fn new() -> Self {
        Context { resources: Vec::new() }
    }

    /// Registers a new resource.
    pub fn register(&mut self, resource: &'a T) -> &mut Self {
        self.resources.push(resource);
        self
    }

    /// Invokes before_checkpoint() on all registered resources in the reverse order of registration.
    pub fn before_checkpoint(&self) -> Result<(), Error> {
        let mut checkpint_errors: Vec<String> = Vec::new();
        for resource in self.resources.iter().rev() {
            let result = resource.before_checkpoint();
            if let Err(err) = result {
                checkpint_errors.push(err.to_string());
            }
        }
        if !checkpint_errors.is_empty() {
            return Err(Box::new(CracError::BeforeCheckpointError(checkpint_errors.join(", "))));
        }
        Ok(())
    }

    /// Invokes after_restore() on all registered resources in the order of registration.
    pub fn after_restore(&self) -> Result<(), Error> {
        let mut restore_errors: Vec<String> = Vec::new();
        for resource in &self.resources {
            let result = resource.after_restore();
            if let Err(err) = result {
                restore_errors.push(err.to_string());
            }
        }
        if !restore_errors.is_empty() {
            return Err(Box::new(CracError::AfterRestoreError(restore_errors.join(", "))));
        }
        Ok(())
    }
}
