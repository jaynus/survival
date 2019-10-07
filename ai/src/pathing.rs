use crate::components::CurrentPathingComponent;
use core::{
    amethyst::{
        core::{math::Point3, SystemDesc},
        ecs::{Entity, Join, Read, ReadExpect, System, SystemData, World, Write, WriteStorage},
        shrev::{EventChannel, ReaderId},
        tiles::{Map, MapStorage, TileMap},
    },
    defs::property::MovementFlags,
    tiles::region::RegionTile,
};
use crossbeam::channel::{Receiver, Sender};
use parking_lot::RwLock;
use rayon::ThreadPool;
use shrinkwraprs::Shrinkwrap;
use smallvec::SmallVec;
use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc},
};

/// Each path has a max 4096 steps for it. This should be enough to traverse a Z-level.
#[derive(Debug)]
pub struct Path {
    pub valid: Arc<AtomicBool>,
    pub total_cost: u32,
    pub path: SmallVec<[u32; 4096]>,
}
impl Default for Path {
    fn default() -> Self {
        Self {
            valid: Arc::new(AtomicBool::new(true)),
            path: SmallVec::default(),
            total_cost: u32::max_value(),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PathingRequestEvent {
    pub entity: Entity,
    pub source: Point3<u32>,
    pub destination: Point3<u32>,
    pub kind: MovementFlags,
}
impl PathingRequestEvent {
    pub fn new(
        entity: Entity,
        source: Point3<u32>,
        destination: Point3<u32>,
        kind: MovementFlags,
    ) -> Self {
        Self {
            entity,
            source,
            destination,
            kind,
        }
    }
}

pub type PathingResponseEvent = PathingResult;

pub type PathingResult = Result<(PathingRequestEvent, Arc<Path>), PathingFailure>;

#[derive(Debug, Clone)]
pub enum PathingFailureKind {
    NoPath,
    Error,
}

#[derive(failure::Fail, Debug, Clone)]
#[fail(display = "Type not implemented.")]
pub struct PathingFailure {
    pub request: PathingRequestEvent,
    pub kind: PathingFailureKind,
}

#[derive(Shrinkwrap, Clone)]
struct TileMapContainer(Option<Arc<RwLock<TileMap<RegionTile>>>>);
impl TileMapContainer {
    pub fn new(map: TileMap<RegionTile>) -> Self {
        Self(Some(Arc::new(RwLock::new(map))))
    }
    pub fn empty() -> Self {
        Self(None)
    }
}

pub struct PathingWorkSystem {
    // We store a copy of the target TileMap to allow multithreading
    pathing_request_reader_id: ReaderId<PathingRequestEvent>,
    channel: (Sender<PathingResult>, Receiver<PathingResult>),
    map: TileMapContainer,
    map_version: u64,
    path_cache: HashMap<PathingRequestEvent, Arc<Path>>,
}

impl<'s> System<'s> for PathingWorkSystem {
    type SystemData = (
        ReadExpect<'s, Arc<ThreadPool>>,
        Read<'s, EventChannel<PathingRequestEvent>>,
        Write<'s, EventChannel<PathingResponseEvent>>,
        WriteStorage<'s, TileMap<RegionTile>>,
        WriteStorage<'s, CurrentPathingComponent>,
    );

    fn run(
        &mut self,
        (_pool, pathing_requests, mut pathing_response, tilemap_storage, mut pathing_storage): Self::SystemData,
    ) {
        // Update the current map
        // TODO: We need to invalidate and sync it
        /*
        if let Some(map) = (&tilemap_storage).join().next() {
            if self.map.is_none() || self.map_version != map.version() {
                log::info!("Re-caching map");
                self.map = TileMapContainer::new(map.clone());
                self.map_version = map.version();
            }
        } else {
            self.map = TileMapContainer::empty();
            return;
        }*/

        // Fire off rayon spawns for pathing requests
        for new_request in pathing_requests.read(&mut self.pathing_request_reader_id) {
            log::trace!("PATHING REQUEST THREAD FIRED");

            let sender = self.channel.0.clone();
            let request = (*new_request).clone();
            //let map = self.map.clone();
            //pool.spawn(move || find_path(request, &map, &sender));
            find_path_sync(request, (&tilemap_storage).join().next().unwrap(), &sender)
        }

        // Collect any available pathing results and fire them off
        let mut msg_queue = Vec::with_capacity(32);
        while let Ok(msg) = self.channel.1.try_recv() {
            let entity = match &msg {
                Ok((request, _)) => {
                    //self.path_cache.insert(request.clone(), path.clone());
                    request.entity
                }
                Err(e) => e.request.entity,
            };

            // TODO: Clear the last path if it existed
            // TODO: for now we just ovewrite it
            pathing_storage
                .insert(entity, CurrentPathingComponent::new(Some(msg.clone())))
                .unwrap();

            msg_queue.push(msg);
        }

        pathing_response.drain_vec_write(&mut msg_queue);

        // TODO: Re-sync map if dirty
    }
}

#[derive(Default)]
pub struct PathingWorkSystemDesc;
impl<'a, 'b> SystemDesc<'a, 'b, PathingWorkSystem> for PathingWorkSystemDesc {
    fn build(self, world: &mut World) -> PathingWorkSystem {
        log::trace!("Setup PathingWorkSystem");
        <PathingWorkSystem as System<'_>>::SystemData::setup(world);

        let pathing_request_reader_id =
            Write::<EventChannel<PathingRequestEvent>>::fetch(world).register_reader();

        PathingWorkSystem {
            pathing_request_reader_id,
            channel: crossbeam::channel::bounded(2048),
            map: TileMapContainer::empty(),
            path_cache: HashMap::with_capacity(1024),
            map_version: 0,
        }
    }
}

fn bail(request: PathingRequestEvent, result_channel: &Sender<PathingResult>) {
    result_channel
        .send(Err(PathingFailure {
            request,
            kind: PathingFailureKind::Error,
        }))
        .unwrap();
}

fn filter_adjacent_tiles<M>(
    (x, y, _z): (u32, u32, u32),
    kind: MovementFlags,
    map: &M,
) -> Vec<((u32, u32, u32), u32)>
where
    M: Map + MapStorage<RegionTile>,
{
    let mut r = SmallVec::<[(u32, u32, u32); 8]>::new();

    if x > 0 && y > 0 {
        //    r.push((x - 1, y - 1, 0));
    }
    if x > 0 {
        r.push((x - 1, y, 0));
        //   r.push((x - 1, y + 1, 0));
    }
    if y > 0 {
        r.push((x, y - 1, 0));
        //     r.push((x + 1, y - 1, 0));
    }

    //   r.push((x + 1, y + 1, 0));
    r.push((x + 1, y, 0));
    r.push((x, y + 1, 0));

    r.into_iter()
        .filter_map(|p| {
            if let Some(coord) = map.encode_raw(&p) {
                if let Some(tile) = map.get_raw(coord) {
                    if tile.passable(kind) {
                        return Some((p, tile.movement_modifier(kind)));
                    }
                } else {
                    log::error!("COULDNT FETCH TILE: {:?} = {:?}", p, coord);
                    log::error!("WHY CANT WE FETCH? {:?}", map.dimensions());
                }
            } else {
                log::error!("COULDNT ENCODE POSITION: {:?}", p);
            }
            None
        })
        .collect()
}

fn find_path(
    request: PathingRequestEvent,
    map: &TileMapContainer,
    result_channel: &Sender<PathingResult>,
) {
    let map = map.as_ref().as_ref().unwrap().read();
    find_path_sync(request, &map, result_channel)
}

fn find_path_sync(
    request: PathingRequestEvent,
    map: &TileMap<RegionTile>,
    result_channel: &Sender<PathingResult>,
) {
    use pathfinding::prelude::{absdiff, astar};

    let mut result = Path::default();

    // TODO: Does the path traverse z levels?
    log::trace!(
        "source= {:?}, target = {:?}",
        request.source,
        request.destination
    );
    if request.source.z != request.destination.z {
        log::error!("source={:?}", request.source);
        log::error!("destination={:?}", request.destination);
        unimplemented!("Z-level pathing is not implemented")
    }

    let start = (request.source.x, request.source.y, request.source.z);

    {
        if !map.get(&request.source).unwrap().passable(request.kind) {
            log::error!("SOURCE TILE IS NOT PASSABLE?!");
        }

        if map
            .get(&request.destination)
            .unwrap()
            .passable(request.kind)
        {
            log::trace!(
                "Path finding work received: {:?} -> {:?}",
                start,
                request.destination
            );

            let path_result = astar(
                &start,
                |&point| filter_adjacent_tiles(point, request.kind, &*map),
                |&(x, y, _)| absdiff(x, request.destination.x) + absdiff(y, request.destination.y),
                |&p| {
                    p == (
                        request.destination.x,
                        request.destination.y,
                        request.destination.z,
                    )
                },
            )
            .map(|r| {
                result.total_cost = r.1;
                r.0.into_iter()
                    .for_each(|step| result.path.push(map.encode_raw(&step).unwrap()));
            });

            log::trace!("FIRING RESULT: {:?}", &result);

            if path_result.is_none() {
                result_channel
                    .send(Err(PathingFailure {
                        request,
                        kind: PathingFailureKind::NoPath,
                    }))
                    .unwrap();
            } else {
                result_channel
                    .send(Ok((request, Arc::new(result))))
                    .unwrap();
            }
        } else {
            result_channel
                .send(Err(PathingFailure {
                    request,
                    kind: PathingFailureKind::NoPath,
                }))
                .unwrap();
        };
    }
}

#[cfg(test)]
mod tests {
    fn find_path_test() {}
}
