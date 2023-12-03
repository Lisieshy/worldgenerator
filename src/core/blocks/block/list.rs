use super::{properties::BlockProperties, Block, BlocksMap};

macro_rules! register_block {
    ($blocks:expr, $block_id:expr, $block_states:expr, $name:expr, $properties:expr) => {
        $blocks.insert(
            $block_id,
            Block {
                block_id: $block_id,
                block_states: $block_states,
                name: $name.to_string(),
                properties: $properties,
            },
        );
    };
}

pub(super) fn create_blocks(blocks: &mut BlocksMap) {
    register_block!(
        blocks,
        0u32,
        0u32..=0u32,
        "air",
        BlockProperties {
            collidable: false,
            opaque: false,
            is_air: true,
        }
    );

    register_block!(
        blocks,
        1u32,
        1u32..=1u32,
        "stone",
        BlockProperties {
            ..Default::default()
        }
    );

    register_block!(
        blocks,
        2u32,
        2u32..=2u32,
        "dirt",
        BlockProperties {
            ..Default::default()
        }
    );

    register_block!(
        blocks,
        3u32,
        3u32..=4u32,
        "grass_block",
        BlockProperties {
            ..Default::default()
        }
    );

    register_block!(
        blocks,
        4u32,
        5u32..=6u32,
        "Oak Log",
        BlockProperties {
            ..Default::default()
        }
    );
}