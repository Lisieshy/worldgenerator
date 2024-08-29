use bevy::{
    prelude::{Handle, Mesh},
    render::primitives::Aabb,
};

use crate::core::enums::Direction;

#[derive(Debug, Clone)]
pub enum BlockModel {
    None,
    Standard,
    Simple(Aabb),
    Complex(Vec<[f32; 3]>),
    Custom {
        collision: Option<Aabb>,
        mesh: Handle<Mesh>,
    },
}

impl BlockModel {
    pub fn mod_mesh_positions(&self, direction: &Direction, pos: &mut [[f32; 3]; 4]) {
        match self {
            Self::Simple(collision)
            | Self::Custom {
                collision: Some(collision),
                ..
            } => {
                Self::mod_bounding_box(direction, *collision, pos);
            }
            Self::Complex(vert_pos) => {
                let side_index = match direction {
                    Direction::Up => 0,
                    Direction::Down => 1,
                    Direction::North => 2,
                    Direction::South => 3,
                    Direction::East => 4,
                    Direction::West => 5,
                };

                for (i, (pos, cpos)) in pos.iter_mut().zip(&Self::CUBE[side_index]).enumerate() {
                    pos.iter_mut()
                        .zip(cpos)
                        .zip(&vert_pos[side_index * 4 + i])
                        .for_each(|((p, c), v)| *p -= c - v);
                }
            }
            _ => {}
        }
    }

    fn mod_bounding_box(direction: &Direction, bounding_box: Aabb, pos: &mut [[f32; 3]; 4]) {
        let [min_x, min_y, min_z] = bounding_box.min().to_array();
        let [max_x, max_y, max_z] = bounding_box.max().to_array();

        match direction {
            Direction::Up | Direction::Down | Direction::East | Direction::West => {
                pos[0][0] += min_x;
                pos[2][0] += min_x;
                pos[1][0] -= 1. - max_x;
                pos[3][0] -= 1. - max_x;
            }
            Direction::North | Direction::South => {
                pos[0][0] += min_x;
                pos[1][0] += min_x;
                pos[2][0] += min_x;
                pos[3][0] += min_x;
            }
        }

        match direction {
            Direction::Up => {
                pos[0][1] -= 1. - max_y;
                pos[1][1] -= 1. - max_y;
                pos[2][1] -= 1. - max_y;
                pos[3][1] -= 1. - max_y;
            }
            Direction::Down => {
                pos[0][1] += min_y;
                pos[1][1] += min_y;
                pos[2][1] += min_y;
                pos[3][1] += min_y;
            }
            Direction::North | Direction::South | Direction::East | Direction::West => {
                pos[0][1] += min_y;
                pos[1][1] += min_y;
                pos[2][1] -= 1. - max_y;
                pos[3][1] -= 1. - max_y;
            }
        }

        match direction {
            Direction::Up | Direction::Down | Direction::North | Direction::South => {
                pos[0][2] += min_z;
                pos[1][2] += min_z;
                pos[2][2] -= 1. - max_z;
                pos[3][2] -= 1. - max_z;
            }
            Direction::East => {
                pos[0][2] += min_z;
                pos[1][2] += min_z;
                pos[2][2] += min_z;
                pos[3][2] += min_z;
            }
            Direction::West => {
                pos[0][2] -= 1. - max_z;
                pos[1][2] -= 1. - max_z;
                pos[2][2] -= 1. - max_z;
                pos[3][2] -= 1. - max_z;
            }
        }
    }

    const CUBE: [[[f32; 3]; 4]; 6] = [
        // Up
        [
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        // Down
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [0.0, 0.0, 1.0],
        ],
        // North
        [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        // South
        [
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
        ],
        // East
        [
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
        // West
        [
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [0.0, 1.0, 0.0],
        ],
    ];
}