//! tracker_core — безынтерфейсная логика Time Tracker (данные, таймеры, бэкапы).
//! Не зависит от Tauri, поэтому тестируется отдельно.

pub mod backups;
pub mod dt;
pub mod logic;
pub mod models;
pub mod report;
pub mod store;

pub use logic::{apply_filter, build_rows, build_totals, TaskFilter, TaskIdGen, Totals, ViewRow};
pub use models::{Entity, Interval, Settings, Task, TaskStatus};
pub use store::Store;