//! Animation state for provider icons.
//!
//! This module contains the [`IconAnimationState`] struct which tracks
//! animation parameters for animated icon rendering, particularly the
//! Codex eye blink animation.

/// Animation state for provider icons.
///
/// This struct allows for animated icon states, particularly for the Codex
/// eye icon which can blink and wiggle.
#[derive(Debug, Clone, Copy)]
pub struct IconAnimationState {
    /// Blink phase (0.0 = open, 1.0 = closed)
    pub blink_phase: f32,
    /// Tilt angle in degrees for "surprise me" mode
    pub tilt_degrees: f32,
    /// Wiggle offset for "surprise me" mode
    pub wiggle_offset: f32,
}

impl Default for IconAnimationState {
    fn default() -> Self {
        Self {
            blink_phase: 0.0,
            tilt_degrees: 0.0,
            wiggle_offset: 0.0,
        }
    }
}

impl IconAnimationState {
    /// Creates a new animation state with default values (eye fully open).
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates an animation state with a specific blink phase.
    pub fn with_blink(blink_phase: f32) -> Self {
        Self {
            blink_phase: blink_phase.clamp(0.0, 1.0),
            ..Default::default()
        }
    }

    /// Returns true if the eye is fully closed.
    pub fn is_closed(&self) -> bool {
        self.blink_phase >= 1.0
    }

    /// Returns true if the eye is fully open.
    pub fn is_open(&self) -> bool {
        self.blink_phase <= 0.0
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_state_default() {
        let state = IconAnimationState::default();
        assert_eq!(state.blink_phase, 0.0);
        assert_eq!(state.tilt_degrees, 0.0);
        assert_eq!(state.wiggle_offset, 0.0);
        assert!(state.is_open());
        assert!(!state.is_closed());
    }

    #[test]
    fn test_animation_state_clamp() {
        // Test that blink phase is clamped
        let state = IconAnimationState::with_blink(2.0);
        assert_eq!(state.blink_phase, 1.0);

        let state = IconAnimationState::with_blink(-1.0);
        assert_eq!(state.blink_phase, 0.0);
    }

    #[test]
    fn test_animation_state_fully_closed() {
        let state = IconAnimationState::with_blink(1.0);
        assert!(state.is_closed());
        assert!(!state.is_open());
    }

    #[test]
    fn test_animation_state_fully_open() {
        let state = IconAnimationState::with_blink(0.0);
        assert!(!state.is_closed());
        assert!(state.is_open());
    }
}
