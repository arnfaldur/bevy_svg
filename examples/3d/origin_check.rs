use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy_svg::prelude::*;

#[path = "../common/lib.rs"]
mod common;

const CAMERA_Z: f32 = -1000.0;

const ANCHORS: [(Origin, Vec2); 5] = [
    (Origin::Center, Vec2::new(0.0, 0.0)),
    (Origin::TopLeft, Vec2::new(-150.0, 150.0)),
    (Origin::TopRight, Vec2::new(150.0, 150.0)),
    (Origin::BottomLeft, Vec2::new(-150.0, -150.0)),
    (Origin::BottomRight, Vec2::new(150.0, -150.0)),
];

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "origin_check".to_string(),
                resolution: (600, 600).into(),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_plugins((common::CommonPlugin, bevy_svg::prelude::SvgPlugin))
        .add_systems(Startup, setup)
        .add_systems(Update, (common::spin, draw_anchors))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let svg: Handle<Svg> = asset_server.load("box.svg");
    commands.spawn(Camera3d::default());
    for (origin, pos) in ANCHORS {
        commands.spawn((
            Svg3d(svg.clone()),
            origin,
            Transform::from_translation(pos.extend(CAMERA_Z)),
            common::Spin,
        ));
    }
}

fn draw_anchors(mut gizmos: Gizmos) {
    for (_, pos) in ANCHORS {
        gizmos.sphere(Isometry3d::from_translation(pos.extend(CAMERA_Z)), 5.0, RED);
    }
}
