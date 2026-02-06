use crate::player::Player;

/// Camera system for tracking the player with dynamic smoothing
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub prev_x: f64,
    pub prev_y: f64,
    screen_width: f64,
    screen_height: f64,
}

impl Camera {
    pub fn new(player_x: f64, player_y: f64, screen_width: f64, screen_height: f64) -> Self {
        let initial_cam_x = player_x - screen_width / 2.0;
        let initial_cam_y = player_y - screen_height / 2.0;
        
        Camera {
            x: initial_cam_x,
            y: initial_cam_y,
            prev_x: initial_cam_x,
            prev_y: initial_cam_y,
            screen_width,
            screen_height,
        }
    }
    
    /// Update camera position with dynamic smoothing based on player speed
    /// Returns the camera movement delta (dx, dy) for parallax calculations
    pub fn update(&mut self, player: &Player, dt: f64) -> (f64, f64) {
        // Calculate target position (center player on screen)
        let target_cam_x = player.x - self.screen_width / 2.0;
        let target_cam_y = player.y - self.screen_height / 2.0;
        
        // Calculate player speed
        let speed = (player.vx * player.vx + player.vy * player.vy).sqrt();
        
        // Dynamic smoothing based on speed:
        // - Speed 1000 px/s: 0.4 (minimum)
        // - Speed 10000 px/s: 0.8 (maximum)
        // - Interpolate between based on current speed
        let min_speed = 1000.0;
        let max_speed = 10000.0;
        let min_smoothing = 0.4;
        let max_smoothing = 0.8;
        let snap_smoothing = 0.9;  // Fast lerp when in control mode
        
        // Calculate smoothing factor
        let smoothing = if player.control_mode {
            // Fast lerp when in control mode
            snap_smoothing
        } else if speed < min_speed {
            // Below minimum speed: use minimum smoothing
            min_smoothing
        } else if speed > max_speed {
            // Above maximum speed: use maximum smoothing
            max_smoothing
        } else {
            // Interpolate between min and max based on speed
            let t = (speed - min_speed) / (max_speed - min_speed);
            min_smoothing + t * (max_smoothing - min_smoothing)
        };
        
        // Store previous position for parallax delta
        let cam_dx = self.x - self.prev_x;
        let cam_dy = self.y - self.prev_y;
        
        // Apply smoothing
        self.x += (target_cam_x - self.x) * smoothing;
        self.y += (target_cam_y - self.y) * smoothing;
        
        // Update previous position
        self.prev_x = self.x;
        self.prev_y = self.y;
        
        (cam_dx, cam_dy)
    }
    
    /// Get current camera X position
    pub fn get_x(&self) -> f64 {
        self.x
    }
    
    /// Get current camera Y position
    pub fn get_y(&self) -> f64 {
        self.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::{Player, MovementMode};

    #[test]
    fn test_camera_initial_position() {
        let player = Player::new();
        let camera = Camera::new(0.0, 0.0, 800.0, 600.0);
        
        // Camera should be centered on player
        assert_eq!(camera.get_x(), -400.0);
        assert_eq!(camera.get_y(), -300.0);
    }

    #[test]
    fn test_camera_follows_moving_player() {
        let mut player = Player::new();
        player.x = 100.0;
        player.y = 50.0;
        
        let mut camera = Camera::new(player.x, player.y, 800.0, 600.0);
        
        // Update camera multiple times to see movement
        camera.update(&player, 0.016);
        camera.update(&player, 0.016);
        camera.update(&player, 0.016);
        
        // Camera should stay centered on player
        assert!((camera.get_x() - (player.x - 400.0)).abs() < 1.0);
        assert!((camera.get_y() - (player.y - 300.0)).abs() < 1.0);
    }

    #[test]
    fn test_camera_returns_movement_delta() {
        let mut player = Player::new();
        let mut camera = Camera::new(player.x, player.y, 800.0, 600.0);
        
        // Move player
        player.x = 100.0;
        
        // Update camera multiple times and get delta
        let (dx1, _dy1) = camera.update(&player, 0.016);
        let (dx2, _dy2) = camera.update(&player, 0.016);
        
        // Should have moved in X direction
        assert!(dx1.abs() > 0.0 || dx2.abs() > 0.0);  // Camera moved
    }

    #[test]
    fn test_control_mode_uses_fast_smoothing() {
        let mut player = Player::new();
        player.control_mode = true;
        player.x = 100.0;
        
        let mut camera = Camera::new(0.0, 0.0, 800.0, 600.0);
        
        // Update with control mode (should use 0.9 smoothing)
        let (dx, _dy) = camera.update(&player, 1.0);
        
        // With 0.9 smoothing, camera should move significantly
        assert!(dx.abs() > 80.0);  // Moved at least 80px toward target
    }
}