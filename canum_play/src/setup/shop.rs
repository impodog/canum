use super::*;

pub(super) struct ShopPlugin;

impl Plugin for ShopPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(enter_shop);
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
        if total_overlap_area < 1e-3 {
            break;
        }
    }
}

#[derive(Component)]
#[require(Transform)]
pub struct ShopItem {
    pub name: String,
    pub price: i32,
    pub range: Rect,
}

const ITEM_RECT_SIZE: Vec2 = Vec2::new(100.0, 50.0);

fn enter_shop(
    event: On<PostStartSession>,
    mut commands: Commands,
    mut state: ResMut<NextState<setup::PlayState>>,
    save: Res<Save>,
) {
    let Some(details) = CONFIG.values.shop.get(&event.fight) else {
        return;
    };
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
    reposition_rects(&mut rects, CONFIG.display.screen_rect);
    for (item, rect) in items.into_iter().zip(rects) {
        let center = rect.center();
        let name = item.item.to_name();
        commands.spawn((
            Transform::from_translation(Vec3::new(center.x, center.y, 15.37)),
            ShopItem {
                name: name.clone(),
                range: rect,
                price: item.price,
            },
            children![(
                Transform::from_translation(Vec3::new(0.0, -16.0, 0.1)),
                Animation::new(name.clone(), vec2(32.0, 32.0))
            )],
        ));
    }
    state.set(setup::PlayState::Shop);
}
