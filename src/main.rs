use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use egui_file::FileDialog;

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin};
use bevy_light_2d::prelude::*;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample4)
        .add_plugins((
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }),
            EguiPlugin,
            Light2dPlugin,
        ))
        .add_systems(Startup, startup)
        .add_systems(Update, draw_ui)
        .run();
}

#[derive(Component)]
struct Level {
    loaded: bool,
    opened_file: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
}

#[derive(Component)]
struct LightName {
    editable_name: String,
}

fn startup(mut commands: Commands) {
    commands.spawn(
        Level {
            loaded: false,
            opened_file: None,
            open_file_dialog: None,
        }
    );

    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 1.0)),
            ..Default::default()
        },
        AmbientLight2d {
            brightness: 1.0,
            ..default()
        },
    ));
}

fn draw_ui(
    mut level: Query<&mut Level>,
    mut camera_transform: Query<(&mut Transform, &mut OrthographicProjection), With<Camera>>,
    mut lights: Query<(&mut PointLight2d, &mut Transform, &mut LightName, &mut Name), Without<Camera>>,
    mut ambient: Query<&mut AmbientLight2d>,
    mut contexts: EguiContexts,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let ctx = contexts.ctx_mut();
    let mut level = level.get_single_mut().unwrap();
    let mut ambient = ambient.get_single_mut().unwrap();
    let (mut camera_transform, mut camera_projection)
        = camera_transform.get_single_mut().unwrap();

    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            if ui.button("Exit").clicked() {
                std::process::exit(0);
            }
            egui::menu::menu_button(ui, "File", |ui| {
                ui.label("Not implemented yet :(");
            });
        });
    });

    let mut open_file = false;
    let mut spawn_light = false;

    let lights_amount = lights.iter().count();
    egui::SidePanel::right("right_panel")
        .resizable(true)
        .default_width(300.0)
        .show(ctx, |ui| {
            if !level.loaded {
                ui.label("Open level png to continue.");
                if ui.button("Open").clicked() {
                    open_file = true;
                }
            }
            ui.collapsing("Camera", |ui| {
                ui.label("Camera position");
                ui.horizontal(|ui| {
                    ui.label("X:");
                    ui.add(egui::Slider::new(&mut camera_transform.translation.x, -256.0..=256.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Y:");
                    ui.add(egui::Slider::new(&mut camera_transform.translation.y, -256.0..=256.0));
                });
                ui.horizontal(|ui| {
                    ui.label("Zoom:");
                    ui.add(egui::Slider::new(&mut camera_projection.scale, -0.1..=5.0));
                });
            });
            ui.collapsing("Lights", |ui| {
                ui.collapsing("Ambient", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Brightness:");
                        ui.add(egui::DragValue::new(&mut ambient.brightness).speed(0.1));
                    });
                    ui.horizontal(|ui| {
                        ui.label("Color:");
                        let srgba = ambient.color.to_srgba();
                        let mut ambient_color = egui::Color32::from_rgb(
                            (srgba.red * 255.0).floor() as u8,
                            (srgba.green * 255.0).floor() as u8,
                            (srgba.blue * 255.0).floor() as u8,
                        );
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            ui.color_edit_button_srgba(&mut ambient_color);
                        });
                        ambient.color = Color::srgb(
                            ambient_color.r() as f32 / 255.0,
                            ambient_color.g() as f32 / 255.0,
                            ambient_color.b() as f32 / 255.0,
                        );
                    });
                });
                if ui.button("Spawn").clicked() {
                    spawn_light = true;
                }
                for (
                    mut light,
                    mut transform,
                    mut edit_name,
                    mut name,
                ) in lights.iter_mut() {
                    ui.collapsing(name.clone().as_str(), |ui| {
                        if ui.text_edit_singleline(&mut edit_name.editable_name).lost_focus() {
                            if edit_name.editable_name != name.as_str() {
                                *name = Name::new(edit_name.editable_name.clone());
                            }
                        }
                        if ui.button("Duplicate").clicked() {
                            let new_name = format!("{} {}", name.as_str(), lights_amount);
                            commands.spawn((
                                light.clone(),
                                transform.clone(),
                                LightName {
                                    editable_name: new_name.to_string().clone(),
                                },
                                Name::new(new_name.to_string().clone()),
                            ));
                        }
                        ui.label("Position");
                        ui.horizontal(|ui| {
                            ui.label("X:");
                            ui.add(egui::DragValue::new(&mut transform.translation.x).speed(0.5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Y:");
                            ui.add(egui::DragValue::new(&mut transform.translation.y).speed(0.5));
                        });
                        ui.separator();
                        ui.horizontal(|ui| {
                            ui.label("Intensity:");
                            ui.add(egui::DragValue::new(&mut light.intensity).speed(0.1));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Radius:");
                            ui.add(egui::DragValue::new(&mut light.radius).speed(0.5));
                        });
                        ui.horizontal(|ui| {
                            ui.label("Falloff:");
                            ui.add(egui::DragValue::new(&mut light.falloff).speed(0.1));
                        });
                        let srgba = light.color.to_srgba();
                        let mut color = egui::Color32::from_rgb(
                            (srgba.red * 255.0).floor() as u8,
                            (srgba.green * 255.0).floor() as u8,
                            (srgba.blue * 255.0).floor() as u8,
                        );
                        ui.horizontal(|ui| {
                            ui.label("Color:");
                            ui.color_edit_button_srgba(&mut color);
                        });
                        light.color = Color::srgb(
                          color.r() as f32 / 255.0,
                          color.g() as f32 / 255.0,
                          color.b() as f32 / 255.0,
                        );
                    });
                }
            });
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();

    if spawn_light {
        commands.spawn((
            PointLight2dBundle {
                point_light: PointLight2d {
                    intensity: 5.0,
                    radius: 32.0,
                    color: Color::WHITE,
                    ..default()
                },
                transform: Transform::from_xyz(
                    0.0, 0.0, 0.0
                ),
                ..default()
            },
            LightName {
                editable_name: format!("Spotlight {}", lights_amount + 1).to_string(),
            },
            Name::new(format!("Spotlight {}", lights_amount + 1)),
        ));
    }

    if open_file {
        let filter = Box::new({
            let ext = Some(OsStr::new("png"));
            move |path: &Path| -> bool { path.extension() == ext }
        });
        let mut dialog = FileDialog::open_file(level.opened_file.clone()).show_files_filter(filter);
        dialog.open();
        level.open_file_dialog = Some(dialog);
    }

    let selected_file = if let Some(dialog) = &mut level.open_file_dialog {
        if dialog.show(ctx).selected() {
            dialog.path().map(|p| p.to_path_buf())
        } else {
            None
        }
    } else {
        None
    };

    if let Some(file_path) = selected_file {
        level.opened_file = Some(file_path.clone());
        level.loaded = true;
        commands.spawn(
            SpriteBundle {
                texture: asset_server.load(file_path.clone()),
                transform: Transform::from_xyz(
                    0.0, 0.0, -1.0
                ),
                ..Default::default()
            },
        );
    }
}
