use float_ord::FloatOrd;
use ilattice::prelude::{Extent, UVec2};
use std::{collections::BTreeMap, sync::RwLock };

use bevy::{
    math::IVec3,
    prelude::Plugin,
};
use once_cell::sync::Lazy;

use self::{
    biomes::{BiomeTerrainGenerator, IntoBoxedTerrainGenerator},
    common::terrain_generate_world_bottom_border,
    noise::{Heightmap, get_chunk_continentalness, get_chunk_erosion, get_chunk_peaks_valleys},
};

use super::{storage::VoxelBuffer, ChunkShape, Voxel, CHUNK_LENGTH_U, CHUNK_LENGTH, materials::{Rock, Air}, material::VoxelMaterial, CHUNK_HEIGHT };

pub mod biomes;

/// noise functions ported over from C / GLSL code
pub mod noise;

/// common functions used by all terrain generators
pub mod common;

// Terrain generator singleton.
pub static TERRAIN_GENERATOR: Lazy<RwLock<TerrainGenerator>> = Lazy::new(Default::default);

#[derive(Default)]
pub struct TerrainGenerator {
    _biomes_map: BTreeMap<FloatOrd<f32>, Box<dyn BiomeTerrainGenerator>>,
    biome_list: Vec<Box<dyn BiomeTerrainGenerator>>,
}

const _BIOME_INVSCALE: f32 = 0.005;

impl TerrainGenerator {
    pub fn _register_biome_generator(
        &mut self,
        chance: f32,
        biome: Box<dyn BiomeTerrainGenerator>,
    ) -> &mut Self {
        self._biomes_map.insert(FloatOrd(chance), biome);
        self
    }

    pub fn register_biome(
        &mut self,
        biome: Box<dyn BiomeTerrainGenerator>,
    ) -> &mut Self {
        self.biome_list.push(biome);
        self
    }

    //returns the biome with the closest temp / humidity
    // #[allow(clippy::borrowed_box)]
    // #[allow(dead_code)]
    // pub fn biome_at_chunk(&self, chunk_key: IVec3) -> &Box<dyn BiomeTerrainGenerator> {
    //     let coords = noise::voronoi(chunk_key.xzy().truncate().as_vec2() * BIOME_INVSCALE);
    //     let p = FloatOrd(noise::rand2to1i(coords));

    //     self.biomes_map
    //         .range(..=p)
    //         .last()
    //         .map_or(self.biomes_map.first_key_value().unwrap().1, |x| x.1)
    // }

    #[allow(clippy::borrowed_box)]
    pub fn _biome_at_xz(
        &self,
        x: i32,
        z: i32,
        humidity_map: &Heightmap<CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        temperature_map: &Heightmap<CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
    ) -> &Box<dyn BiomeTerrainGenerator> {
        let humidity = humidity_map.getf([x as u32, z as u32].into());
        let temperature = temperature_map.getf([x as u32, z as u32].into());

        // info!("humidity: {}, temperature: {}", humidity, temperature);

        match (humidity, temperature) {
            (x, y) if x > 0.8 && y < 0.2 => &self.biome_list[2],
            (x, y) if x > 0.8 && y > 0.2 => &self.biome_list[1],
            _ => &self.biome_list[0],
        }

        // let coods = noise::voronoi(Vec2::new(x as f32, y as f32) * BIOME_INVSCALE);
        // let p = FloatOrd(noise::rand2to1i(coords));

        // self.biomes_mrap
        //     .range(..=p)
        //     .last()
        //     .map_or(self.biomes_map.first_key_value().unwrap().1, |x| x.1)
    }

    pub fn generate(&self, chunk_key: IVec3, buffer: &mut VoxelBuffer<Voxel, ChunkShape>, seed: i32) {
        let continentalness_noise = get_chunk_continentalness(chunk_key, CHUNK_LENGTH_U, seed);
        let erosion_noise = get_chunk_erosion(chunk_key, CHUNK_LENGTH_U, seed);
        let peaks_valleys_noise = get_chunk_peaks_valleys(chunk_key, CHUNK_LENGTH_U, seed);

        // let continentalness = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&continentalness_noise);
        // let erosion = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&erosion_noise);
        // let peaks_valleys = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&peaks_valleys_noise);
        
        let added_noises = (0..continentalness_noise.len()).map(|x| (continentalness_noise[x] + erosion_noise[x] + peaks_valleys_noise[x]) / 3.0).collect::<Vec<f32>>();
        let added_heightmap = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&added_noises);


        let mut surface_level = 64;

        Extent::from_min_and_shape(UVec2::ZERO, UVec2::new(CHUNK_LENGTH, CHUNK_LENGTH))
            .iter2()
            .for_each(|pos| {
                surface_level += added_heightmap.getf(pos.into()) as i32;
                // surface_level += erosion.getf(pos.into()) as i32;

                for h in 0..surface_level {
                    *buffer.voxel_at_mut([pos.x, h as u32, pos.y].into()) = Rock::into_voxel();
                }

                // fill the rest with air
                for h in surface_level..CHUNK_HEIGHT as i32 {
                    *buffer.voxel_at_mut([pos.x, h as u32, pos.y].into()) = Air::into_voxel();
                }

                let biome = &self.biome_list[0];

                biome.carve_terrain_at_xz(chunk_key, pos.x, pos.y, added_heightmap, buffer);

                // for h in surface_level..73 {
                //     *buffer.voxel_at_mut([pos.x, h as u32, pos.y].into()) = Water::into_voxel();
                // }
                surface_level = 64;
            });

        // *buffer.voxel_at_mut([0, 100, 0].into()) = Rock::into_voxel();


        terrain_generate_world_bottom_border(buffer);
    }
}

pub struct TerrainGeneratorPlugin;

impl Plugin for TerrainGeneratorPlugin {
    fn build(&self, _: &mut bevy::prelude::App) {
        TERRAIN_GENERATOR
            .write()
            .unwrap()
            // .register_biome_generator(
            //     0.0f32,
            //     biomes::BasicPlainsBiomeTerrainGenerator.into_boxed_generator(),
            // )
            // .register_biome_generator(
            //     0.8f32,
            //     biomes::BasicDesertBiomeTerrainGenerator.into_boxed_generator(),
            // )
            // .register_biome_generator(
            //     3.21,
            //     biomes::BasicSnowyPlainsBiomeTerrainGenerator.into_boxed_generator(),
            // )
            .register_biome(
                biomes::BasicPlainsBiomeTerrainGenerator.into_boxed_generator(),
            );
            // .register_biome(
            //     biomes::BasicDesertBiomeTerrainGenerator.into_boxed_generator(),
            // )
            // .register_biome(
            //     biomes::BasicSnowyPlainsBiomeTerrainGenerator.into_boxed_generator(),
            // );
    }
}