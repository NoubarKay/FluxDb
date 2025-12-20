use fluxdb_core::engine::database::Database;

pub enum LoadEvent {
    Progress {
        message: String,
        progress: u16,
    },
    Finished(Database),
    Error(String),
}
