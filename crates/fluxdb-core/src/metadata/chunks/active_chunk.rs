

pub struct ActiveChunk {
    // Identity
    pub table_id: u32,
    pub column_ordinal: u16,

    // Physical layout
    pub first_page_id: u32,
    pub pages: Vec<u32>, // chunk may span multiple pages

    // Runtime state
    pub value_count: u32,

    // Runtime stats (finalized on seal)
    // pub min: Option<Value>,
    // pub max: Option<Value>,
}