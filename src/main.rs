
use bevy::prelude::*;

mod menu;
use menu::MainMenuPlugin;

mod countdown;
use countdown::CountDownPlugin;

#[derive(Debug,Clone,Hash,Eq,PartialEq)]
pub enum AppState{
    MainMenu,
    CountDown,
    Game,
    GameOver,
}


fn main() {
    let mut app = App::new();
    app
        .insert_resource(WindowDescriptor {
            width: 700.,
            height: 500.,
            //resizable: false,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_state(AppState::MainMenu)
        .add_plugin(MainMenuPlugin)
        .add_plugin(CountDownPlugin)
        .add_system_set(SystemSet::on_enter(AppState::Game)
            .with_system(setup_game)
        )
        .add_system_set(SystemSet::on_update(AppState::Game)
            .with_system(gravity)
            .with_system(movement)
            .with_system(flap)
            .with_system(walls)
            .with_system(collision)
            .with_system(score_keeper)
        )
        .add_system_set(SystemSet::on_enter(AppState::GameOver)
            .with_system(setup_game_over)
        )
        .add_system_set(SystemSet::on_exit(AppState::GameOver)
            .with_system(game_cleanup)
        )
        .add_system_set(SystemSet::on_update(AppState::GameOver)
            .with_system(game_over))
        .run();
}

struct GameData {
    score: i32,
    score_ent: Entity,
    wall_timer: Timer,
    background_ent: Entity,
}

#[derive(Component)]
struct Player {
    w: f32,
    h: f32,
}

#[derive(Component)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Wall {
    w: f32,
    h: f32,
}

#[derive(Component)]
struct ScoreText;

fn setup_game(
    mut cmd: Commands,
    windows: Res<Windows>,
    asset_server: Res<AssetServer>,
) {
    let window = windows.get_primary();
    if let Some(window) = window {
        cmd.spawn_bundle(OrthographicCameraBundle::new_2d())
            .insert(Transform::from_xyz(window.width()/2.,window.height()/2.,999.9));
    }

    let background_ent = cmd.spawn_bundle(SpriteBundle{
        texture: asset_server.load("background.png"),
        sprite: Sprite {
            custom_size: Some(Vec2::new(700.,500.)),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(350.,250.,0.),
            ..Default::default()
        },
        ..Default::default()
    }).id();

    let player_ent = cmd.spawn_bundle(SpriteBundle {
        texture: asset_server.load("fish.png"),
        sprite: Sprite {
            //color: Color::rgb(0.5,0.5,1.0),
            custom_size: Some(Vec2::new(60.,50.)),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(80.0,250.0,1.0),
            ..Default::default()
        },
        ..Default::default()
    })
        .insert(Player{
            w: 45.,
            h: 35.,
        })
        .insert(Velocity{
            x: 0.,
            y: 0.,
        })
    .id();

    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let score_ent = cmd
        .spawn_bundle(TextBundle{
            style: Style {
                align_self: AlignSelf::FlexEnd,
                position_type: PositionType::Absolute,
                position: Rect {
                    top: Val::Px(5.0),
                    left: Val::Px(15.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            text: Text{
                sections: vec![
                    TextSection {
                        value: "Score: ".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 40.,
                            color: Color::rgb(0.2,0.2,0.2),
                        },
                    },
                    TextSection {
                        value: "".to_string(),
                        style: TextStyle {
                            font: font.clone(),
                            font_size: 40.,
                            color: Color::rgb(0.2,0.2,0.2),
                        },
                    }
                ],
                alignment: Default::default(),

            },
            ..Default::default()
        })
        .insert(ScoreText)
        .id();

    cmd.insert_resource(GameData{
        score: 0,
        score_ent,
        wall_timer: Timer::from_seconds(1.0,true),
        background_ent,
    });

}

fn gravity(
    mut query: Query < &mut Velocity >,
    time: Res<Time>
) {
    for mut vel in query.iter_mut() {
        vel.y -= 400.0*time.delta_seconds_f64() as f32;

    }
}

fn movement(
    mut query: Query<(&mut Transform,&mut Velocity)>,
    time: Res<Time>
) {
    for (mut trans,mut vel) in query.iter_mut() {
        trans.translation.x += vel.x*time.delta_seconds_f64() as f32;
        trans.translation.y += vel.y*time.delta_seconds_f64() as f32;
        if trans.translation.y >= 500.0 {
            trans.translation.y = 500.0;
            vel.y = 0.;
        }
        if trans.translation.y <= 0. {
            trans.translation.y = 0.;
            vel.y = 0.;
        }
        trans.rotation = Quat::from_axis_angle(Vec3::new(0.,0.,1.),vel.y/1000.)
    }
}

fn flap(
    mut query: Query<&mut Velocity,With<Player>>,
    mouse: Res<Input<MouseButton>>
) {
    if mouse.just_pressed(MouseButton::Left) {
        for mut vel in query.iter_mut() {
            vel.y += 400.0
        }
    }
}

fn walls(
    mut cmd: Commands,
    time: Res<Time>,
    mut query: Query<(Entity,&mut Transform),With<Wall>>,
    windows: Res<Windows>,
    mut game_data: ResMut<GameData>,
    asset_server: Res<AssetServer>,
) {
    let dt = time.delta();
    game_data.wall_timer.tick(dt);
    let wall_texture = asset_server.load("pipe.png");
    let window = windows.get_primary().unwrap();
    let gap_min = (200.-game_data.score as f32).max(80.);
    let gap_max = (window.height()-200.-game_data.score as f32).max(80.);
    let gap = quad_rand::gen_range(gap_min,gap_max);
    let top = quad_rand::gen_range(100.,window.height()-100.-gap);
    let bot = window.height()-gap-top;
    if game_data.wall_timer.just_finished() {
        make_wall(
            &mut cmd,
            window.width()-20.,
            window.height()-top,
            60.,
            window.height(),
            wall_texture.clone()
        );
        make_wall(
            &mut cmd,
            window.width()-20.,
            bot-window.height(),
            60.,
            window.height(),
            wall_texture.clone()
        );
    }

    let mut entities_to_remove = Vec::new();
    let mut score_increase = false;
    for (entity,mut trans) in query.iter_mut() {
        trans.translation.x -= 400.*dt.as_secs_f32();
        if trans.translation.x < -60. {
            entities_to_remove.push(entity);
            score_increase = true;
        }
    }
    if score_increase {
        game_data.score += 1;
    }
    for entity in entities_to_remove.iter() {
        cmd.entity(*entity).despawn_recursive();
    }
}

fn make_wall(cmd: &mut Commands, x:f32, y:f32, w:f32, h:f32, texture: Handle<Image>) -> Entity {
    cmd.spawn_bundle(SpriteBundle {
        texture,
        sprite: Sprite {
            custom_size: Some(Vec2::new(w,h)),
            ..Default::default()
        },
        transform: Transform {
            translation: Vec3::new(x+(w/2.),y+(h/2.),1.0),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Wall{w,h}).id()
}

fn collision(
    wall_trans: Query<(&Wall,&Transform)>,
    player_trans: Query<(&Player,&Transform)>,
    mut state: ResMut< State<AppState> >,
) {
    for (player,p_trans) in player_trans.iter() {
        for (wall,w_trans) in wall_trans.iter() {
            let pl = p_trans.translation.x-(player.w/2.);
            let pr = p_trans.translation.x+(player.w/2.);
            let pt = p_trans.translation.y+(player.h/2.);
            let pb = p_trans.translation.y-(player.h/2.);
            let wl = w_trans.translation.x-(wall.w/2.);
            let wr = w_trans.translation.x+(wall.w/2.);
            let wt = w_trans.translation.y+(wall.h/2.);
            let wb = w_trans.translation.y-(wall.h/2.);

            if pl < wr && pr > wl && pt > wb && pb < wt {
                state.set(AppState::GameOver).unwrap();
            }
        }
    }
}

fn game_cleanup(
    mut cmd: Commands,
    player: Query<Entity,With<Player>>,
    wall: Query<Entity,With<Wall>>,
    game_data: Res<GameData>,
    game_over_data: Res<GameOverData>,
) {
    for ent in player.iter() {
        cmd.entity(ent).despawn_recursive();
    }

    for ent in wall.iter() {
        cmd.entity(ent).despawn_recursive();
    }

    cmd.entity(game_data.score_ent).despawn_recursive();
    cmd.entity(game_over_data.game_over_text_entity).despawn_recursive();
    cmd.entity(game_data.background_ent).despawn_recursive();
    
}

fn score_keeper(
    mut score: Query<&mut Text,With<ScoreText>>,
    game_data: Res<GameData>,
) {
    for mut txt in score.iter_mut() {
        let score_txt = format!("{}",game_data.score);
        txt.sections[1].value = score_txt.clone();
    }
}

struct GameOverData{
    game_over_text_entity: Entity,
    reset_timer: Timer,
}

fn setup_game_over(
    mut cmd: Commands,
    asset_server: Res<AssetServer>
) {
    let game_over_text_entity = cmd.spawn_bundle(
        TextBundle {
            style: Style {
                size: Size::new(Val::Undefined, Val::Px(100.0)),
                margin: Rect::all(Val::Auto),
                ..Default::default()
            },
            text: Text::with_section(
                "Game Over",
                TextStyle {
                    font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 100.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                },

                Default::default()
            ),
            //transform: Transform::from_xyz(250.,250.,0.),
            ..Default::default()
        })
        .id();

    cmd.insert_resource(GameOverData{
        game_over_text_entity,
        reset_timer: Timer::from_seconds(0.5,false)
    });
}

fn game_over(
    mut state: ResMut< State<AppState> >,
    mouse: Res<Input<MouseButton>>,
    time: Res<Time>,
    mut game_over_data: ResMut<GameOverData>,
) {
    game_over_data.reset_timer.tick(time.delta());
    if game_over_data.reset_timer.finished() &&
        mouse.just_pressed(MouseButton::Left) {
            state.set(AppState::MainMenu).unwrap();
    }
}
