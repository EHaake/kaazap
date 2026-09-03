//! The campaign map screen: a full-screen, node-based star map you travel
//! Outer Rim → Core. Planets are glyph nodes joined by dotted routes over a
//! slowly twinkling starfield, with a header band and a bottom info panel for
//! the highlighted planet. Mirrors `opponent_select.rs`'s plumbing (a cursor +
//! an owned-outcome enum + `draw(frame, config, …, pulse)` + one app arm), but
//! spans the whole terminal via [`CampaignMapLayout`] and owns its own twinkle
//! clock (kept separate from the selection pulse, per the amended Motion rule).
//! See `specs/009-campaign-map`.

use std::time::Duration;

use crossterm::event::KeyCode;

use crate::{
    STARFIELD_TWINKLE_MS,
    campaign::{PLANETS, Planet, planet_by_id},
    config::Config,
    frame::{Emphasis, Frame, draw_text},
    layout::CampaignMapLayout,
    opponent::opponent_by_id,
    profile::Profile,
};

/// How many stars fill the backdrop. Sparse enough to read as depth, not noise.
const STAR_COUNT: usize = 72;

/// The result of a key on the map: the cursor moved, a match should launch
/// against a planet's next opponent, or the player backed out to the menu. The
/// app performs the launch/transition and plays the matching SFX. Ids are
/// `&'static str` from the `const` graph.
#[derive(Debug)]
pub enum MapOutcome {
    Moved,
    Launch { planet: &'static str, opponent: &'static str },
    Back,
}

/// One backdrop star: a normalized position, a twinkle phase offset, and the
/// glyph it shows at its bright moment.
#[derive(Debug)]
struct Star {
    fx: f32,
    fy: f32,
    phase: f32,
    bright_glyph: char,
}

/// The twinkling starfield: a fixed set of stars (seeded once) and an
/// accumulated time. Each star breathes on its own phase — background only.
#[derive(Debug)]
struct Starfield {
    stars: Vec<Star>,
    elapsed: Duration,
}

impl Starfield {
    fn new() -> Self {
        let stars = (0..STAR_COUNT)
            .map(|_| Star {
                fx: rand::random_range(0.0f32..1.0),
                fy: rand::random_range(0.0f32..1.0),
                phase: rand::random_range(0.0f32..1.0),
                // A minority are the brighter four-point star; the rest are dots.
                bright_glyph: if rand::random_range(0.0f32..1.0) < 0.3 { '✦' } else { '·' },
            })
            .collect();
        Self { stars, elapsed: Duration::ZERO }
    }

    fn tick(&mut self, dt: Duration) {
        self.elapsed += dt;
    }
}

#[derive(Debug)]
pub struct CampaignMapState {
    cursor: usize, // index into PLANETS; kept on an unlocked planet
    stars: Starfield,
}

impl CampaignMapState {
    /// Open the map with the cursor on the first unlocked planet that still has
    /// an opponent to play (the natural "next" node), falling back to the first
    /// unlocked planet.
    pub fn new(profile: &Profile) -> Self {
        let run = profile.campaign();
        let cursor = PLANETS
            .iter()
            .position(|p| run.planet_unlocked(p) && !run.planet_cleared(p))
            .or_else(|| PLANETS.iter().position(|p| run.planet_unlocked(p)))
            .unwrap_or(0);
        Self { cursor, stars: Starfield::new() }
    }

    /// Advance the starfield twinkle. Called from `App::tick` with the frame dt.
    pub fn tick(&mut self, dt: Duration) {
        self.stars.tick(dt);
    }

    /// Handle a key: arrows / `wasd` move the cursor between **unlocked**
    /// planets (rim→core order, wrapping); Enter/Space launches the highlighted
    /// planet's next un-beaten opponent (no-op on a cleared planet); Esc/`x`
    /// backs out. `None` for keys the map ignores.
    pub fn handle_input(&mut self, key: KeyCode, profile: &Profile) -> Option<MapOutcome> {
        let run = profile.campaign();
        let unlocked: Vec<usize> = (0..PLANETS.len())
            .filter(|&i| run.planet_unlocked(&PLANETS[i]))
            .collect();
        if unlocked.is_empty() {
            // The start planet is always unlocked, so this is unreachable — but
            // guard rather than index into an empty list.
            return matches!(key, KeyCode::Esc | KeyCode::Char('x')).then_some(MapOutcome::Back);
        }
        if !unlocked.contains(&self.cursor) {
            self.cursor = unlocked[0];
        }
        let pos = unlocked.iter().position(|&i| i == self.cursor).unwrap();
        let n = unlocked.len();

        match key {
            KeyCode::Up | KeyCode::Left | KeyCode::Char('w') | KeyCode::Char('a') => {
                self.step(unlocked[(pos + n - 1) % n])
            }
            KeyCode::Down | KeyCode::Right | KeyCode::Char('s') | KeyCode::Char('d') => {
                self.step(unlocked[(pos + 1) % n])
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let planet = PLANETS[self.cursor];
                // A cleared planet has no next opponent → nothing to launch.
                run.next_opponent(&planet)
                    .map(|opponent| MapOutcome::Launch { planet: planet.id, opponent })
            }
            KeyCode::Esc | KeyCode::Char('x') => Some(MapOutcome::Back),
            _ => None,
        }
    }

    /// Move the cursor to `target`, returning `Moved` only if it actually
    /// changed — so a single-unlocked-planet map doesn't play a move cue on a
    /// no-op wrap.
    fn step(&mut self, target: usize) -> Option<MapOutcome> {
        (target != self.cursor).then(|| {
            self.cursor = target;
            MapOutcome::Moved
        })
    }

    /// Draw the whole map: starfield, routes, nodes, header, and info panel.
    pub fn draw(&self, frame: &mut Frame, config: &Config, profile: &Profile, pulse: Emphasis) {
        let run = profile.campaign();
        let layout = CampaignMapLayout::new(*config);

        self.draw_starfield(frame, &layout);

        // Routes: a dotted lane from each planet back to every planet it
        // requires. Drawn over the stars, under the nodes.
        for planet in PLANETS {
            let (px, py) = layout.node_pos(planet.fx, planet.fy);
            for req in planet.requires {
                if let Some(pre) = planet_by_id(req) {
                    let (qx, qy) = layout.node_pos(pre.fx, pre.fy);
                    draw_route(frame, (qx, qy), (px, py));
                }
            }
        }

        // Nodes + labels.
        for (i, planet) in PLANETS.iter().enumerate() {
            let (x, y) = layout.node_pos(planet.fx, planet.fy);
            let cursored = i == self.cursor;
            let (glyph, emphasis) = node_style(planet, run, cursored, pulse);
            draw_text(frame, x, y, &glyph.to_string(), emphasis);

            // The cursored planet reads "larger": caps + flanking markers, so
            // the next world to play stands out at a glance (it still breathes
            // with the pulse via `emphasis`).
            let label = if cursored {
                format!("▸ {} ◂", planet.name.to_uppercase())
            } else {
                planet.name.to_string()
            };
            let lx = x.saturating_sub(label.chars().count() / 2);
            draw_text(frame, lx, y + 1, &label, emphasis);
        }

        self.draw_header(frame, &layout, run);
        self.draw_panel(frame, &layout, run);
    }

    fn draw_starfield(&self, frame: &mut Frame, layout: &CampaignMapLayout) {
        let field = layout.field;
        let (w, h) = (field.width(), field.height());
        let period = STARFIELD_TWINKLE_MS as f32;
        let t = self.stars.elapsed.as_millis() as f32;
        for star in &self.stars.stars {
            let x = field.x0 + (star.fx * w.saturating_sub(1) as f32) as usize;
            let y = field.y0 + (star.fy * h.saturating_sub(1) as f32) as usize;
            // Bright for a small, staggered fraction of each star's slow cycle.
            let cycle = (t / period + star.phase).fract();
            let (glyph, emphasis) = if cycle < 0.12 {
                (star.bright_glyph, Emphasis::Normal)
            } else {
                ('·', Emphasis::Muted)
            };
            draw_text(frame, x, y, &glyph.to_string(), emphasis);
        }
    }

    fn draw_header(&self, frame: &mut Frame, layout: &CampaignMapLayout, run: &crate::campaign::CampaignRun) {
        let cleared = PLANETS.iter().filter(|p| run.planet_cleared(p)).count();
        draw_text(frame, layout.header.x0 + 2, layout.header.y0, "CAMPAIGN", Emphasis::Strong);

        let progress = format!("{cleared}/{} worlds cleared", PLANETS.len());
        let px = layout.header.x1.saturating_sub(progress.chars().count() + 1);
        draw_text(frame, px, layout.header.y0, &progress, Emphasis::Normal);

        const AXIS: &str = "Outer Rim  →  The Core";
        let cx = (layout.header.x0 + layout.header.x1) / 2;
        draw_text(frame, cx.saturating_sub(AXIS.chars().count() / 2), layout.header.y0 + 1, AXIS, Emphasis::Muted);
    }

    fn draw_panel(&self, frame: &mut Frame, layout: &CampaignMapLayout, run: &crate::campaign::CampaignRun) {
        let planet = PLANETS[self.cursor];
        let panel = layout.panel;

        let divider: String = "─".repeat(panel.width());
        draw_text(frame, panel.x0, panel.y0, &divider, Emphasis::Muted);

        let x = panel.x0 + 2;
        draw_text(frame, x, panel.y0 + 1, &format!("{}  ·  {}", planet.name, planet.region), Emphasis::Strong);
        draw_text(frame, x, panel.y0 + 2, &opponents_line(&planet, run), Emphasis::Normal);

        let status = if run.run_complete() {
            "Campaign complete — you've reached the Core."
        } else if run.planet_cleared(&planet) {
            "Cleared."
        } else {
            planet.blurb
        };
        draw_text(frame, x, panel.y0 + 3, status, Emphasis::Muted);
        draw_text(frame, x, panel.y0 + 4, "↑/↓ move  ·  Enter play  ·  Esc menu", Emphasis::Muted);
    }
}

/// A planet's node glyph and emphasis for the given run state. The glyph shows
/// state (cleared / open / locked); the cursored planet keeps its glyph but
/// takes the bright pulse so the selection reads at every pulse phase.
fn node_style(planet: &Planet, run: &crate::campaign::CampaignRun, cursored: bool, pulse: Emphasis) -> (char, Emphasis) {
    let (glyph, state_emphasis) = if !run.planet_unlocked(planet) {
        ('◌', Emphasis::Muted)
    } else if run.planet_cleared(planet) {
        ('●', Emphasis::Strong)
    } else {
        ('○', Emphasis::Normal)
    };
    (glyph, if cursored { pulse } else { state_emphasis })
}

/// A one-line summary of a planet's opponents for the info panel: each named,
/// beaten ones marked `✔`, the next-to-play marked `▸`.
fn opponents_line(planet: &Planet, run: &crate::campaign::CampaignRun) -> String {
    let next = run.next_opponent(planet);
    planet
        .opponents
        .iter()
        .map(|&id| {
            let name = opponent_by_id(id).map(|o| o.name).unwrap_or(id);
            if run.is_opponent_beaten(planet.id, id) {
                format!("{name} ✔")
            } else if Some(id) == next {
                format!("▸ {name}")
            } else {
                name.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("    ")
}

/// Draw a dim dotted route between two node cells (endpoints skipped, so it
/// doesn't overdraw the nodes). Clip-safe; diagonals fall out of the linear
/// interpolation.
fn draw_route(frame: &mut Frame, (x0, y0): (usize, usize), (x1, y1): (usize, usize)) {
    let dx = x1 as isize - x0 as isize;
    let dy = y1 as isize - y0 as isize;
    let steps = dx.abs().max(dy.abs());
    for i in 1..steps {
        let t = i as f32 / steps as f32;
        let x = (x0 as f32 + t * dx as f32).round() as usize;
        let y = (y0 as f32 + t * dy as f32).round() as usize;
        draw_text(frame, x, y, "·", Emphasis::Muted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;

    #[test]
    fn cursor_starts_on_the_first_unlocked_unbeaten_planet() {
        let p = Profile::default(); // fresh run: only Cinder unlocked
        let s = CampaignMapState::new(&p);
        assert_eq!(PLANETS[s.cursor].id, "cinder");
    }

    #[test]
    fn navigation_stays_on_unlocked_planets() {
        // Clear Cinder → Ashfall + Drift unlock; The Spindle stays locked.
        let mut p = Profile::default();
        p.campaign_mut().mark_beaten("cinder", "greeb");
        let mut s = CampaignMapState::new(&p);

        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..6 {
            assert!(p.campaign().planet_unlocked(&PLANETS[s.cursor]));
            seen.insert(PLANETS[s.cursor].id);
            s.handle_input(KeyCode::Down, &p);
        }
        assert!(seen.contains("cinder") && seen.contains("ashfall") && seen.contains("drift"));
        assert!(!seen.contains("the-spindle"), "a locked planet must never be selectable");
    }

    #[test]
    fn enter_launches_the_next_unbeaten_opponent() {
        let p = Profile::default();
        let mut s = CampaignMapState::new(&p); // on Cinder
        match s.handle_input(KeyCode::Enter, &p) {
            Some(MapOutcome::Launch { planet, opponent }) => {
                assert_eq!(planet, "cinder");
                assert_eq!(opponent, "greeb");
            }
            other => panic!("expected Launch, got {other:?}"),
        }
    }

    #[test]
    fn enter_on_a_cleared_planet_is_a_no_op() {
        let mut p = Profile::default();
        p.campaign_mut().mark_beaten("cinder", "greeb"); // Cinder cleared
        let mut s = CampaignMapState::new(&p);
        while PLANETS[s.cursor].id != "cinder" {
            s.handle_input(KeyCode::Down, &p);
        }
        assert!(s.handle_input(KeyCode::Enter, &p).is_none());
    }

    #[test]
    fn esc_and_x_back_out() {
        let p = Profile::default();
        let mut s = CampaignMapState::new(&p);
        assert!(matches!(s.handle_input(KeyCode::Esc, &p), Some(MapOutcome::Back)));
        assert!(matches!(s.handle_input(KeyCode::Char('x'), &p), Some(MapOutcome::Back)));
    }
}
