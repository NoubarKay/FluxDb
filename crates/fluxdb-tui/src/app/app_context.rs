use fluxdb_core::general::database::Database;

pub struct AppContext<'a>{
    pub db: Option<&'a Database>
}