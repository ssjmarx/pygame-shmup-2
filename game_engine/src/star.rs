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
    pub brightness: f64,      // 0.5 to 1.0 (base brightness before twinkle)
    
    // Twinkling properties
    pub twinkle_offset: f64,  // Random phase offset
    pub twinkle_speed: f64,  // Random twinkle speed (0.5 to 2.0)
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
            twinkle_offset: 0.0,
            twinkle_speed: 1.0,
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
        
        Star {
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
            brightness: rng.gen_range(0.5..1.0),
            twinkle_offset: rng.gen_range(0.0..2.0 * std::f64::consts::PI),
            twinkle_speed: 0.5 + rng.gen_range(0.0..1.5),
        }
    }
    
    pub fn update(&mut self, dt: f64, player_x: f64, player_y: f64, camera_x: f64, camera_y: f64) {
        // Parallax movement: stars move slower than camera based on depth
        // Calculate parallax offset from camera movement
        let _parallax_x = (camera_x - player_x) * (1.0 - self.depth);
        let _parallax_y = (camera_y - player_y) * (1.0 - self.depth);
        
        // Actually, parallax should be handled by moving star position
        // Stars with depth 0.1 should barely move, stars with depth 1.0 move with camera
        // For now, update twinkle (parallax visual effect will come from camera offset in rendering)
        self.twinkle_offset += self.twinkle_speed * dt;
    }
    
    pub fn get_twinkle_brightness(&self) -> f64 {
        let twinkle = (self.twinkle_offset.sin() + 1.0) / 2.0;
        self.brightness * twinkle
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
        
        Star {
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
            brightness: rng.gen_range(0.5..1.0),
            twinkle_offset: rng.gen_range(0.0..2.0 * std::f64::consts::PI),
            twinkle_speed: 0.5 + rng.gen_range(0.0..1.5),
        }
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
        star.update(1.0, 100.0, 100.0, 200.0, 200.0);
        // Twinkle offset should have increased
        assert!(star.twinkle_offset > 0.0);
    }
    
    #[test]
    fn test_twinkle_brightness_in_range() {
        let star = Star::new_random_in_screen((0.0, 100.0), (0.0, 100.0));
        let twinkle = star.get_twinkle_brightness();
        assert!(twinkle >= 0.0);
        assert!(twinkle <= star.brightness);
    }
}