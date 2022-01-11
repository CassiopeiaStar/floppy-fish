

use bevy::prelude::*;
use crate::AppState;
 
pub struct CountDownPlugin;
impl Plugin for CountDownPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(SystemSet::on_enter(AppState::CountDown).with_system(setup_countdown))
            .add_system_set(SystemSet::on_update(AppState::CountDown).with_system(countdown))
            .add_system_set(SystemSet::on_exit(AppState::CountDown).with_system(countdown_cleanup));
    }
}

struct CountDownTimer(Timer);

#[derive(Component)]
struct CountDown;

struct CountDownData {
    countdown_entity: Entity,
}

fn setup_countdown(mut cmd: Commands,asset_server: Res<AssetServer>) {
    cmd.insert_resource(CountDownTimer(Timer::from_seconds(3.0,false)));
    let countdown_entity = cmd.spawn_bundle(
        TextBundle {
            style: Style {
                size: Size::new(Val::Undefined, Val::Px(300.0)),
                margin: Rect::all(Val::Auto),
                ..Default::default()
            },
            text: Text::with_section(
                "3",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 300.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },

                Default::default()
            ),
            //transform: Transform::from_xyz(250.,250.,0.),
            ..Default::default()
        })
        .insert(CountDown)
        .id();
    cmd.insert_resource(CountDownData{countdown_entity});

}

fn countdown(
    mut state: ResMut< State<AppState> >,
    time: Res<Time>,
    mut timer: ResMut<CountDownTimer>,
    mut query: Query < &mut Text, With<CountDown> >
) {
    if timer.0.tick(time.delta()).just_finished() {
        state.set(AppState::Game).unwrap();
    } else {
        for mut text in query.iter_mut() {
            text.sections[0].value = format!("{}",3 - timer.0.elapsed_secs() as i32);
        }
    }
}

fn countdown_cleanup(
    mut cmd: Commands,
    data: Res<CountDownData>
) {
    cmd.entity(data.countdown_entity).despawn_recursive();
}
