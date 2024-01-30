use bevy::prelude::*;

use crate::SnakeTailNode;

// todo: should this also halve the hunger rate, or no?
pub fn split_snake(
    mut commands: Commands,
    tail_nodes: Query<Entity, With<SnakeTailNode>>,
) {
    let tail_node_count = tail_nodes.iter().count();
    for (i, tail_node) in tail_nodes.iter().enumerate() {
        if i > (tail_node_count / 2) {
            commands.entity(tail_node).despawn();
        }
    }
}
