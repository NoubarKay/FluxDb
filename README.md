> 🚧 **Early Project & Open to Feedback**
>
> FluxDB is an early-stage project and **all feedback is very welcome**! Ideas, critiques, questions, or alternative approaches.
>
> I’m also **new to Rust**, so if you spot something unidiomatic, inefficient, or just plain wrong, please don’t hesitate to call it out.  
> The goal of this project is learning, discussion, and building better systems together.

# FluxDB

**FluxDB** is an experimental **column-oriented analytical database engine** written in **Rust**.

It is built to explore how modern OLAP systems work at a low level — from **pages on disk**, to **column chunks**, to **sequential scans and aggregations**, with an emphasis on **clarity, correctness, and observability**.

FluxDB is not a SQL database and is not intended for production use (yet).  
It is a systems-level project focused on building an analytical engine from first principles to understand how they work.

---

## Why FluxDB?

Most analytical databases hide their internals behind layers of abstraction.

FluxDB does the opposite.

It makes storage layout, chunking, and metadata **explicit and inspectable**, allowing you to understand:
- how columns are written to disk
- how data is segmented into chunks
- how metadata drives reads
- how page-level decisions affect performance

This project is equal parts **learning**, **experimentation**, and **engine design**.

---

## Core Ideas

- **Columnar storage**  
  Data is stored per column, not per row.

- **Chunk-based layout**  
  Each column is split into logical chunks covering contiguous row ranges.

- **Explicit metadata**  
  Schema and storage layout are persisted as metadata.

- **Append-only writes**  
  Data is written sequentially; no in-place updates.

- **Observable internals**  
  The engine is designed to be inspected and debugged, not treated as a black box.

---

## Current Status

### Implemented
- Page allocator and pager
- Slot-based metadata heaps
- Schema catalog (tables & columns)
- Column type system
- Persisted chunk metadata
- Restart-safe metadata loading

### In Progress
- Column data pages
- Column writers (append path)
- Chunk sealing

### Planned
- Sequential column scans
- Aggregations (`COUNT`, `SUM`, `AVG`)
- Chunk pruning with statistics
- Compression (dictionary, RLE)
- Compaction
- Query execution layer

