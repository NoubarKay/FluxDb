use fluxdb_core::engine::database::Database;

pub struct AppContext<'a>{
    pub db: Option<&'a Database>
}