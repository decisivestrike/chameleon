#[derive(Debug, Clone)]
pub enum WorkspaceEvent {
    Create(i32),
    Destroy(i32),
    Activate(i32),
    Deactivate(i32),
}
