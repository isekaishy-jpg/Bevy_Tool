//! Command stack (v0). Expand into delta-based terrain/liquid edits.

#[derive(Debug)]
pub enum Command {
    // TODO: TerrainStroke { ... }
    // TODO: LiquidsPaint { ... }
    // TODO: TransformEdit { ... }
    Noop,
}

#[derive(Debug, Default)]
pub struct CommandStack {
    undo: Vec<Command>,
    redo: Vec<Command>,
}

impl CommandStack {
    pub fn push(&mut self, cmd: Command) {
        self.undo.push(cmd);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}
