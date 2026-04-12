//! Contains the plugin and its helper types.

use bevy::{
    app::{App, Plugin},
    asset::{AssetEvent, AssetId, Assets, Handle},
    ecs::{
        entity::Entity,
        message::MessageReader,
        query::{Added, Changed, Or},
        schedule::{IntoScheduleConfigs, SystemSet},
        system::{Commands, Query, Res, ResMut},
    },
    log::debug,
    math::Vec3,
    mesh::Mesh,
    prelude::Last,
};

#[cfg(feature = "2d")]
use bevy::mesh::Mesh2d;

#[cfg(feature = "3d")]
use bevy::mesh::Mesh3d;

use crate::{
    origin::Origin,
    render::{self, Svg2d, Svg3d},
    svg::Svg,
};

/// Set in which [`Svg`](crate::prelude::Svg2d)s get drawn.
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct SvgSet;

/// A plugin that makes sure your [`Svg`]s get rendered
pub struct SvgRenderPlugin;

impl Plugin for SvgRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Last, svg_mesh_linker.in_set(SvgSet))
            .add_plugins(render::SvgPlugin);
    }
}

#[cfg(feature = "2d")]
#[cfg(not(feature = "3d"))]
type SvgMeshComponents = (
    Entity,
    &'static Handle<Svg>,
    Option<&'static mut Mesh2dHandle>,
    Option<()>,
);
#[cfg(not(feature = "2d"))]
#[cfg(feature = "3d")]
type SvgMeshComponents = (
    Entity,
    &'static Handle<Svg>,
    Option<()>,
    Option<&'static mut Handle<Mesh>>,
);
#[cfg(all(feature = "2d", feature = "3d"))]
type SvgMeshComponents = (
    Entity,
    Option<&'static Svg2d>,
    Option<&'static Svg3d>,
    Option<&'static Origin>,
    Option<&'static mut Mesh2d>,
    Option<&'static mut Mesh3d>,
);

/// Assigns mesh handles to [`Svg2d`]/[`Svg3d`] entities. If the entity carries
/// an [`Origin`], the offset is baked into a per-entity mesh and the component
/// is removed.
fn svg_mesh_linker(
    mut commands: Commands,
    mut svg_events: MessageReader<AssetEvent<Svg>>,
    mut meshes: ResMut<Assets<Mesh>>,
    svgs: Res<Assets<Svg>>,
    mut query: Query<SvgMeshComponents>,
    changed_handles: Query<
        Entity,
        Or<(Changed<Svg2d>, Changed<Svg3d>, Added<Svg2d>, Added<Svg3d>)>,
    >,
) {
    for event in svg_events.read() {
        match event {
            AssetEvent::Added { .. } | AssetEvent::Unused { .. } => (),
            AssetEvent::LoadedWithDependencies { id } | AssetEvent::Modified { id } => {
                let Some(svg) = svgs.get(*id) else {
                    continue;
                };
                debug!("Svg `{}` loaded/modified; linking meshes.", svg.name);
                let canonical = svg.mesh.clone();
                for (entity, _, _, origin, mesh_2d, mesh_3d) in query
                    .iter_mut()
                    .filter(|(_, svg_2d, svg_3d, ..)| matches_svg(*svg_2d, *svg_3d, *id))
                {
                    let new_handle = resolve_mesh_handle(svg, origin, &mut meshes);
                    commands.entity(entity).remove::<Origin>();
                    #[cfg(feature = "2d")]
                    if let Some(mut mesh) = mesh_2d {
                        swap_mesh_handle(&mut mesh.0, new_handle.clone(), &canonical, &mut meshes);
                    }
                    #[cfg(feature = "3d")]
                    if let Some(mut mesh) = mesh_3d {
                        swap_mesh_handle(&mut mesh.0, new_handle, &canonical, &mut meshes);
                    }
                }
            }
            AssetEvent::Removed { id } => {
                for (entity, ..) in query
                    .iter_mut()
                    .filter(|(_, svg_2d, svg_3d, ..)| matches_svg(*svg_2d, *svg_3d, *id))
                {
                    commands.entity(entity).despawn();
                }
            }
        }
    }

    for entity in changed_handles.iter() {
        let Ok((_, svg_2d, svg_3d, origin, mesh_2d, mesh_3d)) = query.get_mut(entity) else {
            continue;
        };
        let Some(handle) = svg_2d.map_or_else(|| svg_3d.map(|x| &x.0), |x| Some(&x.0)) else {
            continue;
        };
        let Some(svg) = svgs.get(handle) else {
            continue;
        };
        debug!("Svg handle for entity `{entity:?}` changed; linking mesh.");
        let canonical = svg.mesh.clone();
        let new_handle = resolve_mesh_handle(svg, origin, &mut meshes);
        commands.entity(entity).remove::<Origin>();

        #[cfg(feature = "2d")]
        if let Some(mut mesh) = mesh_2d {
            swap_mesh_handle(&mut mesh.0, new_handle.clone(), &canonical, &mut meshes);
        }
        #[cfg(feature = "3d")]
        if let Some(mut mesh) = mesh_3d {
            swap_mesh_handle(&mut mesh.0, new_handle, &canonical, &mut meshes);
        }
    }
}

fn matches_svg(svg_2d: Option<&Svg2d>, svg_3d: Option<&Svg3d>, id: AssetId<Svg>) -> bool {
    svg_2d
        .map(|x| x.0.id() == id)
        .or_else(|| svg_3d.map(|x| x.0.id() == id))
        .unwrap_or(false)
}

#[cfg(any(feature = "2d", feature = "3d"))]
fn resolve_mesh_handle(
    svg: &Svg,
    origin: Option<&Origin>,
    meshes: &mut Assets<Mesh>,
) -> Handle<Mesh> {
    origin
        .map(|origin| origin.compute_translation(svg.size))
        .filter(|offset| *offset != Vec3::ZERO)
        .and_then(|offset| {
            meshes
                .get(&svg.mesh)
                .cloned()
                .map(|mesh| meshes.add(mesh.translated_by(offset)))
        })
        .unwrap_or_else(|| svg.mesh.clone())
}

/// Replaces `current` with `new_handle` and removes the displaced mesh asset
/// when it was a per-entity bake (i.e. not the shared canonical).
#[cfg(any(feature = "2d", feature = "3d"))]
fn swap_mesh_handle(
    current: &mut Handle<Mesh>,
    new_handle: Handle<Mesh>,
    canonical: &Handle<Mesh>,
    meshes: &mut Assets<Mesh>,
) {
    if *current == new_handle {
        return;
    }
    let old = std::mem::replace(current, new_handle);
    if old != *canonical {
        meshes.remove(&old);
    }
}
