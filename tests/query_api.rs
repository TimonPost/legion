use legion::prelude::*;
use std::{
    collections::HashMap,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pos(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Rot(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Scale(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Vel(f32, f32, f32);
#[derive(Clone, Copy, Debug, PartialEq)]
struct Accel(f32, f32, f32);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Model(u32);
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
struct Static;

#[test]
fn query_read_entity_data() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = Read::<Pos>::query();

    let mut count = 0;
    for (entity, pos) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        count += 1;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_try_read_entity_data() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();
    world.insert((), Some((Pos(1., 2., 3.),)));
    world.insert((), Some((Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6))));

    let mut query = TryRead::<Rot>::query();
    let rots = query
        .iter(&mut world)
        .map(|x| x.map(|x| *x))
        .collect::<Vec<_>>();
    assert_eq!(rots.iter().filter(|x| x.is_none()).count(), 1);
    assert_eq!(
        rots.iter().cloned().filter_map(|x| x).collect::<Vec<_>>(),
        &[Rot(0.4, 0.5, 0.6)]
    );
}

#[test]
fn query_try_write_entity_data() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();
    world.insert((), Some((Pos(1., 2., 3.),)));
    let entity = world.insert((), Some((Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6))))[0];

    let mut query = TryWrite::<Rot>::query();
    for mut x in query.iter(&mut world).filter_map(|x| x) {
        *x = Rot(9.0, 9.0, 9.0);
    }
    assert_eq!(
        world.get_component::<Rot>(entity).map(|x| *x),
        Some(Rot(9.0, 9.0, 9.0))
    );
}

#[test]
fn query_cached_read_entity_data() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = Read::<Pos>::query(); //.cached();

    let mut count = 0;
    for (entity, pos) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        count += 1;
    }

    assert_eq!(components.len(), count);
}

#[test]
#[cfg(feature = "par-iter")]
fn query_read_entity_data_par() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let count = AtomicUsize::new(0);
    let mut query = Read::<Pos>::query();
    query.par_for_each_chunk(&mut world, |mut chunk| {
        for (entity, pos) in chunk.iter_entities() {
            assert_eq!(expected.get(&entity).unwrap().0, *pos);
            count.fetch_add(1, Ordering::SeqCst);
        }
    });

    assert_eq!(components.len(), count.load(Ordering::SeqCst));
}

#[test]
#[cfg(feature = "par-iter")]
fn query_read_entity_data_par_foreach() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let count = AtomicUsize::new(0);
    let mut query = Read::<Pos>::query();
    query.par_for_each(&mut world, |_pos| {
        count.fetch_add(1, Ordering::SeqCst);
    });

    assert_eq!(components.len(), count.load(Ordering::SeqCst));
}

#[test]
fn query_read_entity_data_tuple() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = <(Read<Pos>, Read<Rot>)>::query();

    let mut count = 0;
    for (entity, (pos, rot)) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        assert_eq!(expected.get(&entity).unwrap().1, *rot);
        count += 1;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_write_entity_data() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = Write::<Pos>::query();

    let mut count = 0;
    for (entity, mut pos) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        count += 1;

        pos.0 = 0.0;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_write_entity_data_tuple() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = <(Write<Pos>, Write<Rot>)>::query();

    let mut count = 0;
    for (entity, (mut pos, mut rot)) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        assert_eq!(expected.get(&entity).unwrap().1, *rot);
        count += 1;

        pos.0 = 0.0;
        rot.0 = 0.0;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_mixed_entity_data_tuple() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = <(Read<Pos>, Write<Rot>)>::query();

    let mut count = 0;
    for (entity, (pos, mut rot)) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        assert_eq!(expected.get(&entity).unwrap().1, *rot);
        count += 1;

        rot.0 = 0.0;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_partial_match() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = <(Read<Pos>, Write<Rot>)>::query();

    let mut count = 0;
    for (entity, (pos, mut rot)) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        assert_eq!(expected.get(&entity).unwrap().1, *rot);
        count += 1;

        rot.0 = 0.0;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_read_shared_data() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    world.insert(shared, components.clone());

    let mut query = Tagged::<Static>::query();

    let mut count = 0;
    for marker in query.iter(&mut world) {
        assert_eq!(Static, *marker);
        count += 1;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_on_changed_first() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = Read::<Pos>::query().filter(changed::<Pos>() | changed::<Rot>());

    let mut count = 0;
    for (entity, pos) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        count += 1;
    }

    assert_eq!(components.len(), count);
}

#[test]
fn query_on_changed_no_changes() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = Read::<Pos>::query().filter(changed::<Pos>());

    let mut count = 0;
    for (entity, pos) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        count += 1;
    }

    assert_eq!(components.len(), count);

    count = 0;
    for (entity, pos) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        count += 1;
    }

    assert_eq!(0, count);
}

#[test]
fn query_on_changed_self_changes() {
    let _ = tracing_subscriber::fmt::try_init();

    let universe = Universe::new();
    let mut world = universe.create_world();

    let shared = (Static, Model(5));
    let components = vec![
        (Pos(1., 2., 3.), Rot(0.1, 0.2, 0.3)),
        (Pos(4., 5., 6.), Rot(0.4, 0.5, 0.6)),
    ];

    let mut expected = HashMap::<Entity, (Pos, Rot)>::new();

    for (i, e) in world.insert(shared, components.clone()).iter().enumerate() {
        if let Some((pos, rot)) = components.get(i) {
            expected.insert(*e, (*pos, *rot));
        }
    }

    let mut query = Write::<Pos>::query().filter(changed::<Pos>());

    let mut count = 0;
    for (entity, mut pos) in query.iter_entities(&mut world) {
        assert_eq!(expected.get(&entity).unwrap().0, *pos);
        *pos = Pos(1., 1., 1.);
        count += 1;
    }

    assert_eq!(components.len(), count);

    count = 0;
    for pos in query.iter(&mut world) {
        assert_eq!(Pos(1., 1., 1.), *pos);
        count += 1;
    }

    assert_eq!(components.len(), count);
}
