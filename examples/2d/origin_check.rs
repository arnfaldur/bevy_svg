use bevy::color::palettes::css::RED;
use bevy::prelude::*;
use bevy_svg::prelude::*;

#[path = "../common/lib.rs"]
mod common;

const ANCHORS: [(Origin, Vec2); 5] = [
    (Origin::Center, Vec2::new(0.0, 0.0)),
    (Origin::TopLeft, Vec2::new(-220.0, 180.0)),
    (Origin::TopRight, Vec2::new(220.0, 180.0)),
    (Origin::BottomLeft, Vec2::new(-220.0, -180.0)),
    (Origin::BottomRight, Vec2::new(220.0, -180.0)),
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
    commands.spawn(Camera2d);
    for (origin, pos) in ANCHORS {
        commands.spawn((
            Svg2d(svg.clone()),
            origin,
            Transform::from_translation(pos.extend(0.0)),
            common::Spin,
        ));
    }
}

fn draw_anchors(mut gizmos: Gizmos) {
    for (_, pos) in ANCHORS {
        gizmos.circle_2d(Isometry2d::from_translation(pos), 4.0, RED);
    }
}
