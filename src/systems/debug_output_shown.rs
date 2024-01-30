use bevy::prelude::*;

use crate::DebugSettings;

pub fn debug_output_shown(debug_settings: Res<DebugSettings>) -> bool {
    debug_settings.output_shown
}
