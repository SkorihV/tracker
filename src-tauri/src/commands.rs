//! Общие типы для команд (state-управление хранилищем, ответ запроса).

use serde::Serialize;
use std::sync::Mutex;

use tracker_core::logic::{Totals, ViewRow};
use tracker_core::store::Store;

pub struct AppStore(pub Mutex<Store>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryView {
    pub rows: Vec<ViewRow>,
    pub totals: Totals,
}