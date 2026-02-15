use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use tachyonfx::pattern::{DiagonalPattern, RadialPattern};
use tachyonfx::{fx, EffectManager, Interpolation};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DeckFx {
    #[default]
    Transition,
    Bg,
}

pub struct EffectRegistry {
    /// Internal effect manager from tachyonfx
    effects: EffectManager<DeckFx>,
}

impl EffectRegistry {
    pub fn new() -> Self {
        Self {
            effects: EffectManager::default(),
        }
    }

    pub fn has_active_effects(&self) -> bool {
        self.effects.is_running()
    }

    pub fn process_effects(
        &mut self,
        duration: impl Into<tachyonfx::Duration>,
        buf: &mut Buffer,
        area: Rect,
    ) {
        self.effects.process_effects(duration.into(), buf, area);
    }

    // effects below

    pub fn clear_effect(&mut self, id: DeckFx) {
        self.effects.cancel_unique_effect(id);
    }

    pub fn register_transition(&mut self) {
        let fx = fx::explode(8.0, 2.0, 120);

        self.effects.add_unique_effect(DeckFx::Transition, fx)
    }

    pub fn register_logo_effect(&mut self) {
        let shimmer = fx::hsl_shift_fg([180.0, 40.0, 0.0], (1400, Interpolation::SineInOut))
            .with_pattern(DiagonalPattern::top_left_to_bottom_right().with_transition_width(10.0));

        let fx = fx::repeating(fx::ping_pong(shimmer));

        self.effects.add_unique_effect(DeckFx::Transition, fx)
    }

    pub fn register_bg_effect(&mut self) {
        let fg_shift = [-330.0, 20.0, 20.0];
        let timer = (1000, Interpolation::SineIn);

        let radial_hsl_xform = fx::hsl_shift_fg(fg_shift, timer)
            .with_pattern(RadialPattern::with_transition((0.5, 0.5), 13.0));

        let fx = fx::repeating(fx::ping_pong(radial_hsl_xform));

        self.effects.add_unique_effect(DeckFx::Bg, fx)
    }
}
