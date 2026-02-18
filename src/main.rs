//! Minimal reproduction of double-free crash when using `observe()` with captured data.
//!
//! The crash occurs when:
//! 1. Entities are spawned with `observe()` closures that capture heap data (e.g., String)
//! 2. Those entities are despawned
//! 3. New entities with similar observers are spawned
//!
//! Expected: Program runs without crashing
//! Actual: Segmentation fault or "double free or corruption (out)" after several clicks
//!
//! Reproduction steps:
//! 1. Run this program: `cargo run --release`
//! 2. Click anywhere in the window repeatedly
//! 3. Observe crash after a few clicks (usually 2-4 clicks)

use bevy::{picking::prelude::*, prelude::*, ui_widgets::observe};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ClickCount>()
        .add_systems(Startup, setup)
        .add_systems(Update, (count_clicks, rebuild_ui))
        .run();
}

#[derive(Resource, Default)]
struct ClickCount(u32);

#[derive(Component)]
struct Container;

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        Container,
    ));
}

fn count_clicks(mouse: Res<ButtonInput<MouseButton>>, mut clicks: ResMut<ClickCount>) {
    if mouse.just_pressed(MouseButton::Left) {
        clicks.0 += 1;
        println!("Click #{}, rebuilding UI...", clicks.0);
    }
}

fn rebuild_ui(
    mut commands: Commands,
    clicks: Res<ClickCount>,
    container: Single<Entity, With<Container>>,
    children: Query<&Children>,
) {
    if !clicks.is_changed() {
        return;
    }

    if let Ok(children) = children.get(*container) {
        for child in children.iter() {
            commands.entity(child).despawn();
        }
    }

    for i in 0..3 {
        let label = format!("Button {}", i + 1);
        commands.spawn((
            Node {
                width: Val::Px(150.0),
                height: Val::Px(40.0),
                margin: UiRect::all(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.7)),
            ChildOf(*container),
            observe({
                let captured = label.clone();
                move |_: On<Pointer<Click>>| {
                    println!("Clicked: {}", captured);
                }
            }),
        ));
    }
}
