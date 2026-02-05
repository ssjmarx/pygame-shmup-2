use rand::Rng;

use crate::config::ProjectileConfig;

/// Projectile type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ProjectileType {
    Tracking,  // Bright blue, large, homing
    AutoFire,  // Yellow, small, straight
}

/// Bullet (projectile)
#[derive(Debug, Clone)]
pub struct Projectile {
    // Position and velocity
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    
    // Properties
    pub projectile_type: ProjectileType,
    pub size: f64,              // Width of pill shape
    pub length: f64,            // Length of pill shape
    pub weight: f64,            // For collision physics
    pub speed: f64,             // Speed scalar
    
    // Lifetime
    pub lifetime: f64,          // Time until expiry (seconds)
    pub max_lifetime: f64,
    
    // Tracking (for tracking bullets only)
    pub target_id: Option<usize>,  // ID of target entity
    pub is_scanning: bool,         // True when scanning for new targets
    pub scan_timer: f64,           // Time between scans
    pub scan_radius: f64,           // How far to scan
    
    // Configuration reference
    config: ProjectileConfig,
}

impl Default for Projectile {
    fn default() -> Self {
        Projectile {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            projectile_type: ProjectileType::AutoFire,
            size: 3.0,
            length: 6.0,
            weight: 1.0,
            speed: 800.0,
            lifetime: 0.0,
            max_lifetime: 3.0,
            target_id: None,
            is_scanning: false,
            scan_timer: 0.0,
            scan_radius: 200.0,
            config: ProjectileConfig::default(),
        }
    }
}

impl Projectile {
    /// Create tracking bullet
    pub fn new_tracking(x: f64, y: f64, angle: f64, 
                      target_id: Option<usize>, config: ProjectileConfig, 
                      player_vx: f64, player_vy: f64) -> Self {
        let vx = angle.cos() * config.tracking_speed + player_vx;
        let vy = angle.sin() * config.tracking_speed + player_vy;
        
        Projectile {
            x,
            y,
            vx,
            vy,
            projectile_type: ProjectileType::Tracking,
            size: config.tracking_size,
            length: config.tracking_length,
            weight: config.tracking_weight,
            speed: config.tracking_speed,
            lifetime: 0.0,
            max_lifetime: config.tracking_lifetime,
            target_id,
            is_scanning: target_id.is_none(),
            scan_timer: config.tracking_scan_interval,
            scan_radius: config.tracking_scan_radius,
            config,
        }
    }
    
    /// Create autofire bullet
    pub fn new_autofire(x: f64, y: f64, angle: f64, config: ProjectileConfig,
                       player_vx: f64, player_vy: f64) -> Self {
        let vx = angle.cos() * config.autofire_speed + player_vx;
        let vy = angle.sin() * config.autofire_speed + player_vy;
        
        Projectile {
            x,
            y,
            vx,
            vy,
            projectile_type: ProjectileType::AutoFire,
            size: config.autofire_size,
            length: config.autofire_length,
            weight: config.autofire_weight,
            speed: config.autofire_speed,
            lifetime: 0.0,
            max_lifetime: config.autofire_lifetime,
            target_id: None,
            is_scanning: false,
            scan_timer: 0.0,
            scan_radius: 0.0,
            config,
        }
    }
    
    pub fn update(&mut self, dt: f64, entities: &[(f64, f64, usize)]) {
        // Update position
        self.x += self.vx * dt;
        self.y += self.vy * dt;
        
        // Update lifetime
        self.lifetime += dt;
        
        // Tracking behavior
        if self.projectile_type == ProjectileType::Tracking {
            if self.target_id.is_some() && !self.is_scanning {
                // Have target, check if still valid
                if let Some(target_id) = self.target_id {
                    // Check if target still exists (caller responsibility)
                    // If not, switch to scanning
                    self.is_scanning = !entities.iter().any(|(_, _, id)| *id == target_id);
                }
            }
            
            if self.is_scanning {
                // Scanning for new targets
                self.scan_timer -= dt;
                if self.scan_timer <= 0.0 {
                    // Scan for nearby entities
                    self.scan_timer = self.config.tracking_scan_interval;
                    
                    if let Some(target) = self.find_nearest_target(entities) {
                        self.target_id = Some(target);
                        self.is_scanning = false;
                    }
                }
            } else if let Some(target_id) = self.target_id {
                // Steer toward target
                if let Some((tx, ty, _)) = entities.iter().find(|(_, _, id)| *id == target_id) {
                    self.steer_toward(*tx, *ty, dt);
                }
            }
        }
    }
    
    fn find_nearest_target(&self, entities: &[(f64, f64, usize)]) -> Option<usize> {
        let mut nearest_id = None;
        let mut nearest_dist = self.scan_radius;
        
        for &(ex, ey, id) in entities {
            let dx = ex - self.x;
            let dy = ey - self.y;
            let dist = (dx * dx + dy * dy).sqrt();
            
            if dist < nearest_dist {
                nearest_dist = dist;
                nearest_id = Some(id);
            }
        }
        
        nearest_id
    }
    
    fn steer_toward(&mut self, target_x: f64, target_y: f64, dt: f64) {
        // Calculate steering force perpendicular to velocity
        let speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
        if speed < 1.0 {
            return;
        }
        
        // Current direction
        let dir_x = self.vx / speed;
        let dir_y = self.vy / speed;
        
        // Direction to target
        let to_target_x = target_x - self.x;
        let to_target_y = target_y - self.y;
        let to_target_dist = (to_target_x * to_target_x + to_target_y * to_target_y).sqrt();
        
        if to_target_dist < 1.0 {
            return;
        }
        
        let to_target_norm_x = to_target_x / to_target_dist;
        let to_target_norm_y = to_target_y / to_target_dist;
        
        // Cross product to get perpendicular direction (2D)
        let perp_x = -dir_y;
        let perp_y = dir_x;
        
        // Dot product to determine which way to turn
        let turn_direction = perp_x * to_target_norm_x + perp_y * to_target_norm_y;
        
        // Apply steering force
        let steer_angle = turn_direction.signum() * self.config.steering_strength * dt;
        
        // Rotate velocity vector
        let cos_s = steer_angle.cos();
        let sin_s = steer_angle.sin();
        let new_vx = self.vx * cos_s - self.vy * sin_s;
        let new_vy = self.vx * sin_s + self.vy * cos_s;
        
        self.vx = new_vx;
        self.vy = new_vy;
    }
    
    pub fn is_expired(&self) -> bool {
        self.lifetime >= self.max_lifetime
    }
    
    pub fn get_rotation(&self) -> f64 {
        if self.vx.abs() < 0.01 && self.vy.abs() < 0.01 {
            0.0
        } else {
            self.vy.atan2(self.vx)
        }
    }
    
    pub fn get_color(&self) -> (u8, u8, u8) {
        match self.projectile_type {
            ProjectileType::Tracking => self.config.tracking_color,
            ProjectileType::AutoFire => self.config.autofire_color,
        }
    }
}

/// Render data for projectile
#[derive(Debug, Clone)]
pub struct ProjectileRenderData {
    pub x: f64,
    pub y: f64,
    pub rotation: f64,
    pub length: f64,
    pub width: f64,
    pub color: (u8, u8, u8),  // RGB
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tracking_bullet_creation() {
        let config = ProjectileConfig::default();
        let proj = Projectile::new_tracking(0.0, 0.0, 0.0, Some(1), config.clone(), 10.0, 0.0);
        
        assert_eq!(proj.projectile_type, ProjectileType::Tracking);
        assert_eq!(proj.size, config.tracking_size);
        assert_eq!(proj.target_id, Some(1));
        assert!(!proj.is_scanning);
    }
    
    #[test]
    fn test_autofire_bullet_creation() {
        let config = ProjectileConfig::default();
        let proj = Projectile::new_autofire(0.0, 0.0, 0.0, config.clone(), 0.0, 0.0);
        
        assert_eq!(proj.projectile_type, ProjectileType::AutoFire);
        assert_eq!(proj.size, config.autofire_size);
        assert_eq!(proj.target_id, None);
    }
    
    #[test]
    fn test_projectile_movement() {
        let config = ProjectileConfig::default();
        let mut proj = Projectile::new_autofire(0.0, 0.0, 0.0, config.clone(), 0.0, 0.0);  // Pointing right
        proj.update(0.1, &[]);  // 0.1 seconds
        
        // Should move right
        assert!(proj.x > 0.0);
        assert_eq!(proj.y, 0.0);
    }
    
    #[test]
    fn test_projectile_expiry() {
        let config = ProjectileConfig::default();
        let mut proj = Projectile::new_autofire(0.0, 0.0, 0.0, config.clone(), 0.0, 0.0);
        proj.update(2.0, &[]);  // 2 seconds
        
        assert!(proj.is_expired());
    }
    
    #[test]
    fn test_tracking_steering() {
        let config = ProjectileConfig::default();
        let mut proj = Projectile::new_tracking(0.0, 0.0, 0.0, Some(1), config.clone(), 0.0, 0.0);
        proj.vx = 100.0;  // Moving right
        proj.vy = 0.0;
        
        let _target = (100.0, 50.0, 1);  // Target at (100, 50)
        proj.steer_toward(100.0, 50.0, 0.1);  // Steer toward target
        
        // Should have turned somewhat
        assert!(proj.vy > 0.0);  // Now moving up slightly
    }
    
    #[test]
    fn test_scanning_mode() {
        let config = ProjectileConfig::default();
        let mut proj = Projectile::new_tracking(0.0, 0.0, 0.0, None, config.clone(), 0.0, 0.0);
        
        // Should start in scanning mode
        assert!(proj.is_scanning);
        
        // Add a nearby entity
        let entities = vec![(10.0, 0.0, 1)];
        proj.scan_timer = 0.0;
        proj.update(0.0, &entities);
        
        // Should find target
        assert_eq!(proj.target_id, Some(1));
        assert!(!proj.is_scanning);
    }
}