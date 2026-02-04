use crate::state::Player;

/// Camera that follows the player
#[derive(Debug)]
pub struct Camera {
    // Camera position (center of view)
    pub x: f64,
    pub y: f64,

    // Screen dimensions
    pub screen_width: f64,
    pub screen_height: f64,

    // Smoothing for camera follow
    pub smoothing: f64,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            x: 0.0,
            y: 0.0,
            screen_width: 800.0,
            screen_height: 600.0,
            smoothing: 0.1, // Lerp factor (0.0 = no movement, 1.0 = instant)
        }
    }

    /// Track player position with smoothing
    pub fn track_player(&mut self, player: &Player) {
        // Target position: center camera on player
        let target_x = player.x - self.screen_width / 2.0;
        let target_y = player.y - self.screen_height / 2.0;

        // Smooth interpolation (lerp)
        self.x += (target_x - self.x) * self.smoothing;
        self.y += (target_y - self.y) * self.smoothing;
    }

    /// Get current camera offset (for rendering)
    pub fn get_offset(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    /// Convert world coordinates to screen coordinates
    pub fn world_to_screen(&self, world_x: f64, world_y: f64) -> (f64, f64) {
        let screen_x = world_x - self.x;
        let screen_y = world_y - self.y;
        (screen_x, screen_y)
    }

    /// Convert screen coordinates to world coordinates
    pub fn screen_to_world(&self, screen_x: f64, screen_y: f64) -> (f64, f64) {
        let world_x = screen_x + self.x;
        let world_y = screen_y + self.y;
        (world_x, world_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_follows_player() {
        let mut camera = Camera::new();
        let mut player = Player::new();
        player.x = 500.0;
        player.y = 300.0;

        // Track multiple times to reach target (accounting for smoothing)
        for _ in 0..100 {
            camera.track_player(&player);
        }

        // Camera should center on player
        let expected_x = 500.0 - 400.0; // 800/2
        let expected_y = 300.0 - 300.0; // 600/2
        // Use approximate equality due to floating point and smoothing
        assert!((camera.x - expected_x).abs() < 0.1);
        assert!((camera.y - expected_y).abs() < 0.1);
    }

    #[test]
    fn test_camera_smoothing() {
        let mut camera = Camera::new();
        let player = Player {
            x: 1000.0,
            y: 600.0,
            ..Default::default()
        };

        // First update
        camera.track_player(&player);
        let x1 = camera.x;

        // Second update (should be closer to target)
        camera.track_player(&player);
        let x2 = camera.x;

        // Should move toward target but not instantly
        assert_ne!(x1, x2);
        assert!(x1 < x2); // Moving toward 600.0
    }
}