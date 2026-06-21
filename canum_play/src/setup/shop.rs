use bevy::text::TextBounds;

use super::*;

pub(super) struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_shop);
        app.add_systems(
            FixedPreUpdate,
            update_select_item.run_if(in_state(setup::PlayState::Shop)),
        );
        app.add_systems(
            FixedUpdate,
            (update_item_outline, update_item_description).run_if(in_state(setup::PlayState::Shop)),
        );
        app.add_systems(
            FixedPostUpdate,
            purchase_item.run_if(in_state(setup::PlayState::Shop)),
        );
        app.add_systems(
            FixedLast,
            quit_shop.run_if(in_state(setup::PlayState::Shop)),
        );
        app.add_systems(FixedPreUpdate, init_shop_indicator);
        app.add_observer(transition_enter_shop)
            .add_observer(enter_shop)
            .add_observer(update_purchase_item);
    }
}

/// Randomly repositions rectangles within `bound` while greedily minimizing overlaps.
pub fn reposition_rects(rects: &mut [Rect], bound: Rect) {
    // Cache dimensions to avoid repeated subtractions
    let dims: Vec<(f32, f32)> = rects
        .iter()
        .map(|r| (r.max.x - r.min.x, r.max.y - r.min.y))
        .collect();

    // Initial random placement (fully contained within bounds)
    for (i, r) in rects.iter_mut().enumerate() {
        let (w, h) = dims[i];
        // Ensure placement range is valid even if a rect exactly matches bound size
        let max_x = (bound.max.x - w).max(bound.min.x);
        let max_y = (bound.max.y - h).max(bound.min.y);

        r.min.x = rand::random_range(bound.min.x..=max_x);
        r.min.y = rand::random_range(bound.min.y..=max_y);
        r.max.x = r.min.x + w;
        r.max.y = r.min.y + h;
    }

    // Iterative repulsion to minimize overlaps
    let max_iters = 20;
    let repulsion_damping = 0.65; // 0.5 = split evenly, >0.5 = aggressive separation

    for _ in 0..max_iters {
        let mut total_overlap_area = 0.0;

        for i in 0..rects.len() {
            for j in (i + 1)..rects.len() {
                // Calculate axis-aligned overlap
                let ox = (rects[i].max.x.min(rects[j].max.x) - rects[i].min.x.max(rects[j].min.x))
                    .max(0.0);
                let oy = (rects[i].max.y.min(rects[j].max.y) - rects[i].min.y.max(rects[j].min.y))
                    .max(0.0);

                if ox > 0.0 && oy > 0.0 {
                    total_overlap_area += ox * oy;

                    // Resolve along the axis with smaller penetration (more efficient)
                    if ox < oy {
                        let push = ox * repulsion_damping * 0.5;
                        let dir = if rects[i].min.x < rects[j].min.x {
                            -1.0
                        } else {
                            1.0
                        };
                        rects[i].min.x += push * dir;
                        rects[i].max.x += push * dir;
                        rects[j].min.x -= push * dir;
                        rects[j].max.x -= push * dir;
                    } else {
                        let push = oy * repulsion_damping * 0.5;
                        let dir = if rects[i].min.y < rects[j].min.y {
                            -1.0
                        } else {
                            1.0
                        };
                        rects[i].min.y += push * dir;
                        rects[i].max.y += push * dir;
                        rects[j].min.y -= push * dir;
                        rects[j].max.y -= push * dir;
                    }
                }
            }
        }

        // Clamp all rectangles back inside the bounding box
        for (i, r) in rects.iter_mut().enumerate() {
            let (w, h) = dims[i];
            r.min.x = r.min.x.clamp(bound.min.x, bound.max.x - w);
            r.min.y = r.min.y.clamp(bound.min.y, bound.max.y - h);
            r.max.x = r.min.x + w;
            r.max.y = r.min.y + h;
        }

        // Early exit if overlaps are negligible
        if total_overlap_area < 0.1 {
            break;
        }
    }
}

#[derive(Component)]
#[require(Transform, Visibility, SelectingItem)]
pub struct ShopItem {
    pub item: canum_res::config::ShopItem,
    pub price: i32,
    pub range: Rect,
}

const ITEM_RECT_SIZE: Vec2 = Vec2::new(150.0, 70.0);

#[derive(Resource)]
struct ShopDrawing {
    active: Handle<ColorMaterial>,
    inactive: Handle<ColorMaterial>,
    not_enough: Handle<ColorMaterial>,
    fill_back: Handle<ColorMaterial>,
    rect_outline: Handle<Mesh>,
    rect_fill: Handle<Mesh>,
}
#[derive(Component, Clone, Deref, DerefMut, Default)]
struct SelectingItem(pub bool);

/// Marks the outline mesh, to update color accordingly.
#[derive(Component, Default)]
struct ItemOutline;

/// Marks the descriptions so that it only shows up when item is selected.
#[derive(Component, Default)]
struct ItemDescription;

fn setup_shop(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let drawing = ShopDrawing {
        active: materials.add(ColorMaterial::from_color(Srgba::rgb_u8(10, 240, 100))),
        inactive: materials.add(ColorMaterial::from_color(Color::BLACK)),
        not_enough: materials.add(ColorMaterial::from_color(Srgba::rgb_u8(240, 50, 10))),
        fill_back: materials.add(ColorMaterial::from_color(Srgba::rgb_u8(20, 20, 20))),
        rect_outline: meshes.add(Rectangle::new(ITEM_RECT_SIZE.x, ITEM_RECT_SIZE.y).to_ring(2.0)),
        rect_fill: meshes.add(Rectangle::new(ITEM_RECT_SIZE.x, ITEM_RECT_SIZE.y)),
    };
    commands.insert_resource(drawing);
}

/// Send this to start a transition sequence to shop.
#[derive(Event, Debug, Clone)]
pub struct EnterShop {
    pub fight: String,
}

fn transition_enter_shop(
    event: On<EnterShop>,
    mut commands: Commands,
    q_camera: Query<Entity, With<canum_res::camera::PixelCamera>>,
) {
    let Ok(camera) = q_camera.single() else {
        return;
    };
    commands.spawn((
        ChildOf(camera),
        crate::setup::cutscene::PureColorCutscene {
            transition: canum_fx::transition::PureColor {
                destroy: None,
                duration: Duration::from_secs_f32(0.8),
                color: Color::srgb_u8(20, 20, 0),
                remove_self: true,
            },
            fight: event.fight.clone(),
        },
    ));
}

fn enter_shop(
    event: On<StartSessionFirst>,
    mut commands: Commands,
    mut state: ResMut<NextState<setup::PlayState>>,
    save: Res<Save>,
    drawing: Res<ShopDrawing>,
    lang: Res<Lang>,
    fonts: Res<canum_res::PixelFonts>,
) {
    let Some(details) = CONFIG.values.shop.get(&event.fight) else {
        return;
    };
    if !details.background.is_empty() {
        commands.spawn((
            canum_res::background::Background::new(CONFIG.display.screen_size),
            Animation::new(details.background.clone(), CONFIG.display.screen_size)
                .with_color(Color::linear_rgba(1.0, 1.0, 1.0, 0.8)),
        ));
    }
    let mut items = Vec::new();
    for item in details.items.iter() {
        if !save.progress.has_shop_item(&item.item) {
            items.push(item);
        }
    }
    let mut rects = Vec::new();
    rects.resize(
        items.len(),
        Rect::from_center_size(Vec2::ZERO, ITEM_RECT_SIZE),
    );
    {
        let mut spawning_range = CONFIG.display.screen_rect;
        spawning_range.max.y -= 50.0;
        reposition_rects(&mut rects, spawning_range);
    }
    for (item, rect) in items.into_iter().zip(rects) {
        let center = rect.center();
        let name = item.item.to_name();
        let short_title = lang.get(&format!("{name}_Short"));
        let desc = lang.get(&format!("{name}_ShopDesc"));
        let displace = rect.center().x.signum() * -ITEM_RECT_SIZE.x;
        commands.spawn((
            Transform::from_translation(Vec3::new(center.x, center.y, 15.37)),
            ShopItem {
                item: item.item.clone(),
                range: rect,
                price: item.price,
            },
            crate::setup::SessionOnly,
            SelectingItem(false),
            children![
                (
                    Transform::from_translation(Vec3::new(0.0, 16.0, 0.1)),
                    Animation::new(name.clone(), vec2(32.0, 32.0))
                ),
                (
                    Transform::from_translation(Vec3::new(0.0, -16.0, 0.0)),
                    canum_res::ImageFontPreRenderedText::default(),
                    canum_res::ImageFontText::default()
                        .text(format!("{} ({}G)", short_title, item.price))
                        .font(fonts.normal.clone()),
                    TextBounds {
                        width: Some(ITEM_RECT_SIZE.x),
                        height: None
                    },
                ),
                (
                    Transform::from_translation(Vec3::new(displace, 0.0, 1.0)),
                    ItemDescription,
                    Mesh2d(drawing.rect_fill.clone()),
                    MeshMaterial2d(drawing.fill_back.clone()),
                    Visibility::Hidden,
                    children![(
                        canum_res::ImageFontPreRenderedText::default(),
                        canum_res::ImageFontText::default()
                            .text(desc)
                            .font(fonts.normal.clone())
                    )]
                ),
                (
                    Transform::from_translation(Vec3::new(0.0, 0.0, -0.1)),
                    ItemOutline,
                    Mesh2d(drawing.rect_outline.clone()),
                    MeshMaterial2d(drawing.inactive.clone()),
                )
            ],
        ));
    }
    state.set(setup::PlayState::Shop);
}

fn update_select_item(
    mut q_item: Query<(&ShopItem, &mut SelectingItem)>,
    q_transform: Query<&GlobalTransform>,
    player: Option<Res<crate::player::PrimaryPlayer>>,
) {
    let Some(player) = player else {
        return;
    };
    let Ok(player_transform) = q_transform.get(player.0) else {
        return;
    };
    let player_position = player_transform.translation().xy();
    q_item.par_iter_mut().for_each(|(item, mut selecting)| {
        let current_selecting = item.range.contains(player_position);
        if **selecting ^ current_selecting {
            **selecting = current_selecting;
        }
    });
}

fn update_item_outline(
    mut q_outline: Query<(&mut MeshMaterial2d<ColorMaterial>, &ChildOf), With<ItemOutline>>,
    q_selecting: Query<(Ref<SelectingItem>, &ShopItem)>,
    drawing: Res<ShopDrawing>,
    save: Res<Save>,
) {
    q_outline.par_iter_mut().for_each(|(mut material, parent)| {
        let Ok((selecting, item)) = q_selecting.get(parent.0) else {
            return;
        };
        if selecting.is_changed() {
            **material = if selecting.0 {
                if save.progress.coins >= item.price {
                    drawing.active.clone()
                } else {
                    drawing.not_enough.clone()
                }
            } else {
                drawing.inactive.clone()
            };
        }
    });
}

fn update_item_description(
    mut q_description: Query<(&mut Visibility, &ChildOf), With<ItemDescription>>,
    q_selecting: Query<Ref<SelectingItem>>,
) {
    q_description
        .par_iter_mut()
        .for_each(|(mut visibility, parent)| {
            let Ok(selecting) = q_selecting.get(parent.0) else {
                return;
            };
            if selecting.is_changed() {
                *visibility = if selecting.0 {
                    Visibility::Inherited
                } else {
                    Visibility::Hidden
                };
            }
        });
}

/// Input handlers send this when an item is purchased.
#[derive(Event, Debug, Clone)]
struct PurchaseItem {
    item: canum_res::config::ShopItem,
    price: i32,
    target_entity: Entity,
}

/// Informs other systems that a purchase has been made.
#[derive(Event, Debug, Clone)]
pub struct PurchaseItemSuccess {
    pub item: canum_res::config::ShopItem,
}

fn purchase_item(
    key: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    q_item: Query<(Entity, &ShopItem)>,
    q_transform: Query<&GlobalTransform>,
    player: Option<Res<crate::player::PrimaryPlayer>>,
    q_override: Query<(), With<crate::controls::OverrideMainControls>>,
) {
    if q_override.iter().next().is_some() {
        return;
    }
    let ok = key.just_pressed(KeyCode::Enter);
    if ok {
        let Some(player) = player else {
            return;
        };
        let Ok(player_transform) = q_transform.get(player.0) else {
            return;
        };
        let player_position = player_transform.translation().xy();
        for (target_entity, item) in q_item.iter() {
            if item.range.contains(player_position) {
                commands.trigger(PurchaseItem {
                    item: item.item.clone(),
                    price: item.price,
                    target_entity,
                });
            }
        }
    }
}

fn update_purchase_item(
    event: On<PurchaseItem>,
    mut save: ResMut<Save>,
    mut commands: Commands,
    q_transform: Query<&GlobalTransform>,
) {
    if save.progress.has_shop_item(&event.item) {
        return;
    }
    if save.progress.coins < event.price {
        commands.spawn(Sound::new("Ui_EquipError"));
        return;
    };
    let Ok(transform) = q_transform.get(event.target_entity) else {
        return;
    };
    let translation = transform.translation();
    save.progress.coins -= event.price;
    save.progress.insert_shop_item(event.item.clone());
    commands.entity(event.target_entity).despawn();
    commands.spawn(Sound::new("Ui_Purchase"));
    commands.spawn((
        crate::setup::SessionOnly,
        canum_fx::splash::GravitySplash {
            color: Color::srgba_u8(255, 200, 0, 128),
            duration: Duration::from_secs_f32(0.8),
            start_speed: 300.0,
            number: 20,
        },
        Transform::from_translation(translation),
    ));
    commands.trigger(PurchaseItemSuccess {
        item: event.item.clone(),
    });
}

fn quit_shop(
    mut commands: Commands,
    key: Res<ButtonInput<KeyCode>>,
    q_camera: Query<Entity, With<canum_res::camera::PixelCamera>>,
) {
    if key.any_just_pressed([KeyCode::KeyS, KeyCode::Escape, KeyCode::Backspace]) {
        let Ok(camera) = q_camera.single() else {
            return;
        };
        commands.spawn((
            ChildOf(camera),
            crate::setup::cutscene::PureColorCutscene {
                transition: canum_fx::transition::PureColor {
                    destroy: None,
                    duration: Duration::from_secs_f32(0.8),
                    color: Color::srgb_u8(200, 200, 200),
                    remove_self: true,
                },
                fight: "LobbySelect".to_owned(),
            },
        ));
    }
}

#[derive(Component)]
pub struct ShopIndicator;

fn init_shop_indicator(
    mut commands: Commands,
    mut q_indicator: Query<Entity, Added<ShopIndicator>>,
    fonts: Res<canum_res::PixelFonts>,
    lang: Res<Lang>,
) {
    for entity in q_indicator.iter_mut() {
        commands.entity(entity).insert((
            canum_res::ImageFontPreRenderedText::default(),
            canum_res::ImageFontText::default()
                .text(lang.get("Ui_ShopIndicator_Keyboard").to_owned())
                .font_height(14.0)
                .font(fonts.normal.clone()),
        ));
    }
}
