use crate::app::types::DiagramTab;
use crate::history::History;
use diagram_models::document::DiagramDocument;
use dioxus::prelude::*;
use std::collections::HashMap;

#[derive(Clone, Copy)]
pub struct TabsState {
    pub active_tab_id: Signal<String>,
    pub background_tabs: Signal<HashMap<String, DiagramTab>>,
    pub tab_names: Signal<Vec<(String, String)>>,
}

pub fn use_tabs_logic(
    _doc_signal: Signal<DiagramDocument>,
    _history_signal: Signal<History>,
) -> TabsState {
    let active_tab_id = use_signal(|| "default".to_string());
    let background_tabs = use_signal(HashMap::<String, DiagramTab>::new);
    let tab_names = use_signal(|| vec![("default".to_string(), "Diagram 1".to_string())]);

    TabsState {
        active_tab_id,
        background_tabs,
        tab_names,
    }
}

pub fn switch_tab(
    target_id: String,
    state: &mut TabsState,
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
) {
    if target_id == *state.active_tab_id.read() {
        return;
    }

    let current_id = state.active_tab_id.read().clone();
    let current_name = state
        .tab_names
        .read()
        .iter()
        .find(|(id, _)| id == &current_id)
        .map_or_else(|| "Unknown".to_string(), |(_, name)| name.clone());

    state.background_tabs.write().insert(
        current_id.clone(),
        DiagramTab {
            id: current_id,
            name: current_name,
            doc: doc_signal.read().clone(),
            history: history_signal.read().clone(),
        },
    );

    if let Some(target_tab) = state.background_tabs.write().remove(&target_id) {
        *doc_signal.write() = target_tab.doc;
        *history_signal.write() = target_tab.history;
    } else {
        *doc_signal.write() = DiagramDocument::default();
        *history_signal.write() = History::new();
    }

    state.active_tab_id.set(target_id);
}

pub fn add_tab(
    state: &mut TabsState,
    doc_signal: Signal<DiagramDocument>,
    history_signal: Signal<History>,
) {
    let new_id = uuid::Uuid::new_v4().to_string();
    let new_index = state.tab_names.read().len() + 1;
    let new_name = format!("Diagram {new_index}");
    state
        .tab_names
        .write()
        .push((new_id.clone(), new_name.clone()));

    state.background_tabs.write().insert(
        new_id.clone(),
        DiagramTab {
            id: new_id.clone(),
            name: new_name,
            doc: DiagramDocument::default(),
            history: History::new(),
        },
    );

    switch_tab(new_id, state, doc_signal, history_signal);
}

pub fn close_tab(
    close_id: String,
    state: &mut TabsState,
    mut doc_signal: Signal<DiagramDocument>,
    mut history_signal: Signal<History>,
) {
    let is_active = close_id == *state.active_tab_id.read();

    state.tab_names.write().retain(|(id, _)| id != &close_id);
    state.background_tabs.write().remove(&close_id);

    if state.tab_names.read().is_empty() {
        let default_id = "default".to_string();
        state
            .tab_names
            .write()
            .push((default_id.clone(), "Diagram 1".to_string()));
        *doc_signal.write() = DiagramDocument::default();
        *history_signal.write() = History::new();
        state.active_tab_id.set(default_id);
        return;
    }

    if is_active {
        let next_id = state.tab_names.read()[0].0.clone();
        if let Some(target_tab) = state.background_tabs.write().remove(&next_id) {
            *doc_signal.write() = target_tab.doc;
            *history_signal.write() = target_tab.history;
        }
        state.active_tab_id.set(next_id);
    }
}
