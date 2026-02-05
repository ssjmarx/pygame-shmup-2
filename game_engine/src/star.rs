use rand::Rng;

/// Star shape variants
#[derive(Debug, Clone, Copy)]
pub enum StarShape {
    Circle,
    FourPoint,   // ✦
    SixPoint,    // ✶
}

/// Star color palette
#[derive(Debug, Clone, Copy)]
pub enum StarColor {
    White,
    LightBlue,
    Cyan,
    LightPurple,
    Pink,
    PaleYellow,
}

/// Star with parallax and visual properties
#[derive(Debug, Clone)]
pub struct Star {
    // World position
    pub x: f64,
    pub y: f64,
    
    // Parallax properties
    pub depth: f64,  // 0.1 to 1.0, affects movement speed
    
    // Visual properties (all randomized at creation)
    pub shape: StarShape,     // Circle, FourPoint, or SixPoint
    pub color: StarColor,     // Random color from palette
    pub size: f64,          // 1.0 to 4.0 pixels
    pub brightness: f64,      // Base brightness (used to calculate twinkle range)
    
    // Twinkling properties (instant flip between two levels)
    pub twinkle_enabled: bool,  // Whether this star twinkles at all (only 25% of stars)
    pub twinkle_low: f64,    // Lower brightness level when twinkle is "off"
    pub twinkle_high: f64,   // Higher brightness level when twinkle is "on"
    pub twinkle_current: f64, // Current brightness (either low or high)
    pub twinkle_timer: f64,   // Time until next flip
    pub twinkle_interval: f64,  // How long to wait between flips (0.1-0.4 seconds)
}

impl Default for Star {
    fn default() -> Self {
        Star {
            x: 0.0,
            y: 0.0,
            depth: 0.5,
            shape: StarShape::Circle,
            color: StarColor::White,
            size: 2.0,
            brightness: 0.75,
            twinkle_enabled: false,
            twinkle_low: 0.8,
            twinkle_high: 1.0,
            twinkle_current: 0.9,
            twinkle_timer: 0.2,
            twinkle_interval: 0.2,
        }
    }
}

impl Star {
    pub fn new_random_in_screen(x_range: (f64, f64), y_range: (f64, f64)) -> Self {
        let mut rng = rand::thread_rng();
        
        // Star size: smaller average, wider range
        // 70% chance of small stars (0.3-2.0), 30% chance of larger stars (2.0-5.0)
        let size = if rng.gen_bool(0.7) {
            0.3 + rng.gen_range(0.0..1.7)  // 0.3 to 2.0
        } else {
            2.0 + rng.gen_range(0.0..3.0)  // 2.0 to 5.0
        };
        
        let mut star = Star {
            x: rng.gen_range(x_range.0..x_range.1),
            y: rng.gen_range(y_range.0..y_range.1),
            depth: rng.gen_range(0.1..1.0),
            shape: [  // Randomly choose shape
                StarShape::Circle,
                StarShape::FourPoint,
                StarShape::SixPoint,
            ][rng.gen_range(0..3)],
            color: [  // Randomly choose color
                StarColor::White,
                StarColor::LightBlue,
                StarColor::Cyan,
                StarColor::LightPurple,
                StarColor::Pink,
                StarColor::PaleYellow,
            ][rng.gen_range(0..6)],
            size,
            brightness: 0.7 + rng.gen_range(0.0..0.3),  // 0.7 to 1.0
            twinkle_enabled: false,
            twinkle_low: 0.0,
            twinkle_high: 0.0,
            twinkle_current: 0.0,
            twinkle_interval: 0.2,
            twinkle_timer: 0.1,
        };
        
        // Only 10% of stars twinkle
        star.twinkle_enabled = rng.gen_bool(0.1);
        // Create two brightness levels for instant twinkling (reverted to original range)
        star.twinkle_low = 0.2 + rng.gen_range(0.0..0.3);  // 0.2 to 0.5 (dim)
        star.twinkle_high = 0.7 + rng.gen_range(0.0..0.3);  // 0.7 to 1.0 (bright)
        star.twinkle_current = star.twinkle_high;  // Start at high
        star.twinkle_interval = 0.05 + rng.gen_range(0.0..0.15);  // 0.05 to 0.2 seconds between flips (faster)
        star.twinkle_timer = 0.05 + rng.gen_range(0.0..0.15);  // Initial timer random (faster)
        
        star
    }
    
    pub fn update(&mut self, dt: f64, player_x: f64, player_y: f64, camera_x: f64, camera_y: f64) {
        // Only update twinkling if enabled
        if self.twinkle_enabled {
            self.twinkle_timer -= dt;
            if self.twinkle_timer <= 0.0 {
                // Flip to opposite brightness level
                if self.twinkle_current == self.twinkle_low {
                    self.twinkle_current = self.twinkle_high;
                } else {
                    self.twinkle_current = self.twinkle_low;
                }
                // Reset timer with interval
                self.twinkle_timer = self.twinkle_interval;
            }
        }
    }
    
    pub fn get_twinkle_brightness(&self) -> f64 {
        if self.twinkle_enabled {
            self.twinkle_current  // Return current brightness (instant flip)
        } else {
            self.brightness  // Non-twinkling stars always at base brightness
        }
    }
    
    pub fn to_render_data(&self, camera_x: f64, camera_y: f64) -> StarRenderData {
        StarRenderData {
            x: self.x,
            y: self.y,
            shape: self.shape,
            color: self.color,
            size: self.size,
            twinkle: self.get_twinkle_brightness(),
        }
    }
    
    pub fn new_random_at_edge(camera_x: f64, camera_y: f64) -> Self {
        let mut rng = rand::thread_rng();
        
        // Choose which edge to spawn on (top, bottom, left, or right)
        // Spawn 200-300px out to prevent pop-in while allowing stars to enter view
        let edge = rng.gen_range(0..4);
        let (x, y) = match edge {
            0 => {
                // Top edge - spawn 200-300px above screen
                (
                    camera_x + rng.gen_range(-100.0..900.0),
                    camera_y - rng.gen_range(200.0..300.0)
                )
            },
            1 => {
                // Bottom edge - spawn 200-300px below screen
                (
                    camera_x + rng.gen_range(-100.0..900.0),
                    camera_y + 600.0 + rng.gen_range(200.0..300.0)
                )
            },
            2 => {
                // Left edge - spawn 200-300px left of screen
                (
                    camera_x - rng.gen_range(200.0..300.0),
                    camera_y + rng.gen_range(-100.0..700.0)
                )
            },
            _ => {
                // Right edge - spawn 200-300px right of screen
                (
                    camera_x + 800.0 + rng.gen_range(200.0..300.0),
                    camera_y + rng.gen_range(-100.0..700.0)
                )
            }
        };
        
        let mut star = Star {
            x, y,
            depth: rng.gen_range(0.1..1.0),
            shape: [  // Randomly choose shape
                StarShape::Circle,
                StarShape::FourPoint,
                StarShape::SixPoint,
            ][rng.gen_range(0..3)],
            color: [  // Randomly choose color
                StarColor::White,
                StarColor::LightBlue,
                StarColor::Cyan,
                StarColor::LightPurple,
                StarColor::Pink,
                StarColor::PaleYellow,
            ][rng.gen_range(0..6)],
            size: {
                // Same weighted size distribution
                if rng.gen_bool(0.7) {
                    0.3 + rng.gen_range(0.0..1.7)  // 0.3 to 2.0
                } else {
                    2.0 + rng.gen_range(0.0..3.0)  // 2.0 to 5.0
                }
            },
            brightness: 0.7 + rng.gen_range(0.0..0.3),  // 0.7 to 1.0
            twinkle_enabled: false,
            twinkle_low: 0.0,
            twinkle_high: 0.0,
            twinkle_current: 0.0,
            twinkle_interval: 0.2,
            twinkle_timer: 0.1,
        };
        
        // Only 10% of stars twinkle
        star.twinkle_enabled = rng.gen_bool(0.1);
        // Create two brightness levels for instant twinkling (reverted to original range)
        star.twinkle_low = 0.2 + rng.gen_range(0.0..0.3);  // 0.2 to 0.5 (dim)
        star.twinkle_high = 0.7 + rng.gen_range(0.0..0.3);  // 0.7 to 1.0 (bright)
        star.twinkle_current = star.twinkle_high;  // Start at high
        star.twinkle_interval = 0.05 + rng.gen_range(0.0..0.15);  // 0.05 to 0.2 seconds between flips (faster)
        star.twinkle_timer = 0.05 + rng.gen_range(0.0..0.15);  // Initial timer random (faster)
        
        star
    }
}

#[derive(Debug, Clone)]
pub struct StarRenderData {
    pub x: f64,
    pub y: f64,
    pub shape: StarShape,
    pub color: StarColor,
    pub size: f64,
    pub twinkle: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_star_parallax() {
        let mut star = Star::new_random_in_screen((0.0, 100.0), (0.0, 100.0));
        star.depth = 0.5;  // Moves at 50% of camera speed
        star.twinkle_enabled = true;  // Enable twinkling for test
        star.twinkle_interval = 0.5;  // Set specific interval
        star.twinkle_timer = 0.3;   // Set specific timer (will flip in 0.3s)
        star.update(1.0, 100.0, 100.0, 200.0, 200.0);
        // Timer should have wrapped and reset to interval after flipping
        assert_eq!(star.twinkle_timer, 0.5);  // Should reset to interval
    }
    
    #[test]
    fn test_twinkle_brightness_in_range() {
        let star = Star::new_random_in_screen((0.0, 100.0), (0.0, 100.0));
        let twinkle = star.get_twinkle_brightness();
        // Twinkle should be either low or high value
        assert!(twinkle >= star.twinkle_low || twinkle <= star.twinkle_high);
    }
    
    #[test]
    fn test_twinkle_flips() {
        let mut star = Star::new_random_in_screen((0.0, 100.0), (0.0, 100.0));
        star.twinkle_enabled = true;  // Enable twinkling for test
        star.twinkle_interval = 0.2;  // Set fixed interval
        star.twinkle_timer = 0.1;      // Will flip in 0.1s
        star.twinkle_current = star.twinkle_low;
        
        let initial = star.get_twinkle_brightness();
        assert_eq!(initial, star.twinkle_low);
        
        // Update past timer
        star.update(0.2, 0.0, 0.0, 0.0, 0.0);
        
        // Should have flipped to high
        assert_eq!(star.get_twinkle_brightness(), star.twinkle_high);
    }
}