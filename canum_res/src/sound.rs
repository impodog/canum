use std::{collections::HashMap, sync::Mutex, time::Duration};

use bevy::{audio::Volume, prelude::*};

use crate::config::CONFIG;

/// Stores handles to loaded sound for better efficiency.
#[derive(Resource, Default, Debug, Deref, DerefMut)]
pub(crate) struct LoadedSounds(HashMap<String, Handle<AudioSource>>);

/// Stores sound playback info.
#[derive(Component, Debug, Clone, Default)]
pub struct Sound {
    pub name: String,
    pub paused: bool,
    pub base_volume: f32,
}
impl Sound {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            paused: false,
            base_volume: 1.0,
        }
    }
    pub fn paused(mut self) -> Self {
        self.paused = true;
        self
    }
    pub fn with_base_volume(mut self, base_volume: f32) -> Self {
        self.base_volume = base_volume;
        self
    }
    pub fn with_volume_add(mut self, decibels: f32) -> Self {
        self.base_volume += decibels;
        self
    }
}

pub(super) fn start_playing_sound(
    commands: ParallelCommands,
    mut q_sound: Query<(Entity, &mut Sound), Added<Sound>>,
    asset_server: Res<AssetServer>,
    loaded: ResMut<LoadedSounds>,
) {
    let loaded = Mutex::new(loaded);
    q_sound.par_iter_mut().for_each(|(entity, mut sound)| {
        let Some(details) = CONFIG.assets.sounds.get(&sound.name) else {
            if sound.name != "Empty" {
                log::warn!(
                    "Unable to find sound named {}. No sound will play.",
                    sound.name
                );
            }
            commands.command_scope(|mut commands| {
                commands.entity(entity).despawn();
            });
            return;
        };
        let handle = loaded
            .lock()
            .unwrap()
            .entry(sound.name.clone())
            .or_insert_with(|| asset_server.load(details.path.as_path()))
            .clone();
        sound.base_volume += details.volume;
        commands.command_scope(move |mut commands| {
            commands.entity(entity).insert((
                AudioPlayer::new(handle),
                PlaybackSettings {
                    mode: if details.loop_point.is_some() {
                        bevy::audio::PlaybackMode::Loop
                    } else {
                        bevy::audio::PlaybackMode::Despawn
                    },
                    start_position: details.loop_point.map(Duration::from_secs_f32),
                    paused: sound.paused,
                    volume: Volume::Decibels(sound.base_volume),
                    ..Default::default()
                },
            ));
        });
    });
}
pub(super) fn update_sound(mut q_sound: Query<(&Sound, &AudioSink), Changed<Sound>>) {
    q_sound.par_iter_mut().for_each(|(sound, audio)| {
        if sound.paused {
            audio.pause();
        } else {
            audio.play();
        }
    });
}

/// Times the fading effect.
#[derive(Component, Debug, Deref, DerefMut)]
pub(super) struct FadeTimer(Timer);
impl Default for FadeTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(0.5, TimerMode::Once))
    }
}

/// Marks a sound to fade in. The component removes itself when done.
#[derive(Component, Default)]
#[require(FadeTimer, Sound)]
pub struct FadeIn;

/// Marks a music to fade out and despawn.
#[derive(Component, Default)]
#[require(FadeTimer, Sound)]
pub struct FadeOut;

pub(super) fn fade_in(
    mut commands: Commands,
    mut q_sound: Query<(Entity, &mut FadeTimer, &mut AudioSink, &Sound), With<FadeIn>>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut audio, sound) in q_sound.iter_mut() {
        audio.set_volume(Volume::SILENT.fade_towards(
            Volume::Decibels(sound.base_volume),
            timer.elapsed_secs() / timer.duration().as_secs_f32(),
        ));
        timer.tick(time.delta());
        if timer.is_finished() {
            commands
                .entity(entity)
                .remove::<FadeTimer>()
                .remove::<FadeIn>();
        }
    }
}

pub(super) fn fade_out(
    mut commands: Commands,
    mut q_sound: Query<(Entity, &mut FadeTimer, &mut AudioSink, &Sound), With<FadeOut>>,
    time: Res<Time>,
) {
    for (entity, mut timer, mut audio, sound) in q_sound.iter_mut() {
        audio.set_volume(Volume::Decibels(sound.base_volume).fade_towards(
            Volume::SILENT,
            timer.elapsed_secs() / timer.duration().as_secs_f32(),
        ));
        timer.tick(time.delta());
        if timer.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Marks single-track background music.
#[derive(Component, Default)]
#[require(FadeIn)]
pub struct Music;

pub(super) fn change_music(mut commands: Commands, q_music: Query<(Entity, Ref<Music>)>) {
    let any_added = q_music.iter().any(|(_, music)| music.is_added());
    if any_added {
        for (entity, music) in q_music.iter() {
            if !music.is_added() {
                commands.entity(entity).insert(FadeOut);
            }
        }
    }
}
