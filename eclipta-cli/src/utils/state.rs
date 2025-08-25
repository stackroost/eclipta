use serde::{Serialize, Deserialize};
use std::{fs, path::PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AttachmentRecord {
    pub name: String,
    pub kind: String,
    pub trace_category: Option<String>,
    pub trace_name: Option<String>,
    pub pinned_prog: Option<PathBuf>,
    pub pinned_maps: Vec<PathBuf>,
    pub pid: u32,
    pub created_at: i64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct State {
    pub attachments: Vec<AttachmentRecord>,
}

pub fn load_state(path: &PathBuf) -> State {
    if let Ok(bytes) = fs::read(path) {
        if let Ok(s) = serde_json::from_slice::<State>(&bytes) {
            return s;
        }
    }
    State::default()
}

pub fn save_state(path: &PathBuf, mut st: State) -> std::io::Result<()> {
    if let Some(dir) = path.parent() { fs::create_dir_all(dir)?; }
    // Dedup by (name, kind, pinned_prog)
    st.attachments.sort_by(|a,b| a.name.cmp(&b.name));
    let bytes = serde_json::to_vec_pretty(&st).unwrap();
    fs::write(path, bytes)
} 