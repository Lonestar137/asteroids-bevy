use crate::prelude::*;
use bevy::prelude::*;

#[derive(Clone, Debug, Resource)]
pub struct GameState {
    pub tutorial: bool,
    // pub quests: Quests,
    pub checkpoint_notification: bool,
    pub level: u32,
    pub checkpoint: Option<Box<GameState>>,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            tutorial: false,
            // quests: Quests::default(),
            level: 1,
            checkpoint_notification: false,
            checkpoint: None,
        }
    }
}

impl GameState {
    pub fn checkpoint(&mut self) {
        self.checkpoint_notification = true;
        self.checkpoint = Some(Box::new(self.clone()));
    }

    pub fn restore_checkpoint(&mut self) -> bool {
        if let Some(checkpoint) = self.checkpoint.take() {
            let GameState { level, .. } = *self;
            *self = *checkpoint.clone();
            self.checkpoint = Some(checkpoint);
            self.checkpoint_notification = false;
            self.level = level;
            true
        } else {
            false
        }
    }
}
