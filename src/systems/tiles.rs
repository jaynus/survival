use core::{
    amethyst::{
        core::{SystemDesc, Transform},
        ecs::{
            storage::ComponentEvent, Join, ReadStorage, System, SystemData, World, Write,
            WriteStorage,
        },
        shrev::ReaderId,
        tiles::{Map, TileMap},
    },
    components::{SpatialComponent, TilePosition},
    hibitset::{BitSet, BitSetLike},
    tiles::{region::RegionTile, TileEntityStorage},
};

#[derive(Default)]
pub struct TileEntitySystem {
    reader_id: Option<ReaderId<ComponentEvent>>,
    dirty: BitSet,
}
impl<'s> System<'s> for TileEntitySystem {
    type SystemData = (
        Write<'s, TileEntityStorage>,
        ReadStorage<'s, Transform>,
        WriteStorage<'s, TilePosition>,
        ReadStorage<'s, SpatialComponent>,
        ReadStorage<'s, TileMap<RegionTile>>,
    );

    fn run(
        &mut self,
        (
            mut entity_storage,
            transform_storage,
            mut tilepos_storage,
            spatial_storage,
            tilemap_storage,
        ): Self::SystemData,
    ) {
        self.dirty.clear();

        // Get dirty transforms
        for event in transform_storage
            .channel()
            .read(self.reader_id.as_mut().unwrap())
        {
            match event {
                ComponentEvent::Modified(id) | ComponentEvent::Inserted(id) => {
                    self.dirty.add(*id);
                }
                _ => {}
            }
        }

        if let Some(tilemap) = (&tilemap_storage).join().next() {
            // Join anything that has a tileposition and a transform
            // AND has been marke dirty. thats all we care about updating.
            for (id, tile_position, transform, spatial) in (
                &self.dirty,
                &mut tilepos_storage,
                &transform_storage,
                &spatial_storage,
            )
                .join()
            {
                let old_position = *tile_position;
                tile_position.0 = tilemap.to_tile(transform.translation()).unwrap();

                for pos in spatial.occupies_tiles(&old_position.0).iter() {
                    let morton = tilemap.encode(&pos).unwrap();
                    let mut empty = false;
                    if let Some(entry) = entity_storage.get_mut(&morton) {
                        entry.remove(id);
                        if entry.is_empty() {
                            empty = true;
                        }
                    }
                    if empty {
                        entity_storage.remove(&morton);
                    }
                }

                for pos in spatial.occupies_tiles(&tile_position.0).iter() {
                    entity_storage
                        .entry(tilemap.encode(&pos).unwrap())
                        .or_insert_with(BitSet::new)
                        .add(id);
                }
            }
        }
    }
}

impl<'a, 'b> SystemDesc<'a, 'b, TileEntitySystem> for TileEntitySystem {
    fn build(self, world: &mut World) -> Self {
        <Self as System<'_>>::SystemData::setup(world);
        let reader_id = Some(WriteStorage::<Transform>::fetch(world).register_reader());

        Self {
            reader_id,
            ..Default::default()
        }
    }
}
