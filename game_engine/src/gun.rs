use crate::config::GunConfig;

/// Gun mounted on player ship
#[derive(Debug, Clone)]
pub struct Gun {
    // Position relative to player center (anchored to ship vertices)
    pub offset_x: f64,
    pub offset_y: f64,
    
    // Current rotation (radians)
    pub angle: f64,
    
    // Target rotation (mouse position)
    pub target_angle: Option<f64>,
    
    // Recoil tracking (per-gun)
    pub recoil_accumulated: f64,
    
    // Autofire timing
    pub last_autofire_time: f64,
    pub autofire_cooldown_current: f64,
    
    // Configuration
    config: GunConfig,
}

impl Gun {
    pub fn new(offset_x: f64, offset_y: f64, config: GunConfig) -> Self {
        let initial_angle = offset_y.atan2(offset_x);
        let initial_cooldown = config.autofire_cooldown_start;
        
        Gun {
            offset_x,
            offset_y,
            angle: initial_angle,  // Start pointing in base direction
            target_angle: None,
            config,
            recoil_accumulated: 0.0,
            last_autofire_time: 0.0,
            autofire_cooldown_current: initial_cooldown,
        }
    }
    
    /// Update gun tracking toward target with ship rotation
    pub fn update_tracking_with_ship(
        &mut self,
        _player_facing: f64,
        _player_rotation: f64,
        _player_vx: f64,
        _player_vy: f64,
        _player_speed: f64,
        _movement_compensation: f64,
        dt: f64
    ) {
        self.update_tracking(_player_facing, 0.0, 0.0, 0.0, 1.0, dt);
    }
    
    /// Update gun tracking toward target
    pub fn update_tracking(
        &mut self, 
        _player_angle: f64,
        _player_vx: f64,
        _player_vy: f64,
        _player_speed: f64,
        _movement_compensation: f64,
        dt: f64
    ) {
        // Decay recoil
        if self.recoil_accumulated > 0.0 {
            self.recoil_accumulated -= self.config.recoil_decay_rate * dt;
            if self.recoil_accumulated < 0.0 {
                self.recoil_accumulated = 0.0;
            }
        }
        
        // Decay autofire spool-down (2 seconds to return to base)
        if self.autofire_cooldown_current > self.config.autofire_cooldown_min {
            self.autofire_cooldown_current -= self.config.autofire_spool_down_rate * dt;
            if self.autofire_cooldown_current < self.config.autofire_cooldown_min {
                self.autofire_cooldown_current = self.config.autofire_cooldown_min;
            }
        }
        
        // If no target, return
        let target = match self.target_angle {
            Some(angle) => angle,
            None => return,
        };
        
        // Direct rotation toward target - no constraints, full 360° rotation
        let shortest_diff = shortest_angle_diff(self.angle, target);
        let max_rotation = self.config.rotation_speed * dt;
        
        if shortest_diff.abs() <= max_rotation {
            self.angle = target;
        } else {
            self.angle += shortest_diff.signum() * max_rotation;
        }
    }
    
    /// Apply recoil to this gun
    pub fn add_recoil(&mut self, amount: f64) {
        let random_offset = (rand::random::<f64>() - 0.5) * self.config.recoil_random_offset_max;
        self.recoil_accumulated += random_offset.abs();
        
        // Clamp to maximum (5 stacks at max autofire)
        self.recoil_accumulated = self.recoil_accumulated.min(self.config.recoil_stack_multiplier * amount);
    }
    
    /// Get current angle including recoil
    pub fn get_firing_angle(&self) -> f64 {
        self.angle + (rand::random::<f64>() - 0.5) * self.recoil_accumulated * self.config.recoil_angle_multiplier
    }
    
    /// Update autofire timing (call when autofiring)
    pub fn update_autofire(&mut self, current_time: f64) -> bool {
        if current_time - self.last_autofire_time >= self.autofire_cooldown_current {
            self.last_autofire_time = current_time;
            return true;  // Ready to fire
        }
        false
    }
    
    /// Spool up autofire rate (call when holding mouse)
    pub fn spool_up_autofire(&mut self, dt: f64) {
        // Increase from 0.5s to 0.1s over 2 seconds
        if self.autofire_cooldown_current > self.config.autofire_cooldown_min {
            self.autofire_cooldown_current -= self.config.autofire_spool_rate * dt;
            if self.autofire_cooldown_current < self.config.autofire_cooldown_min {
                self.autofire_cooldown_current = self.config.autofire_cooldown_min;
            }
        }
    }
    
    /// Set target angle for this gun
    pub fn set_target_angle(&mut self, angle: f64) {
        self.target_angle = Some(angle);
    }
}

/// Calculate shortest angle difference between two angles
fn shortest_angle_diff(from: f64, to: f64) -> f64 {
    let mut diff = to - from;
    while diff > std::f64::consts::PI {
        diff -= 2.0 * std::f64::consts::PI;
    }
    while diff < -std::f64::consts::PI {
        diff += 2.0 * std::f64::consts::PI;
    }
    diff
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_gun_creation() {
        let config = GunConfig::default();
        let gun = Gun::new(-7.5, 10.0, config.clone());
        assert_eq!(gun.offset_x, -7.5);
        assert_eq!(gun.offset_y, 10.0);
        assert_eq!(gun.angle, -std::f64::consts::PI / 2.0);
    }
    
    #[test]
    fn test_gun_tracking() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone());
        gun.set_target_angle(0.0);  // Point right
        
        gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 0.016);
        
        // Should rotate toward target
        assert!(gun.angle.abs() < 0.1);
    }
    
    #[test]
    fn test_full_rotation() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone());
        gun.set_target_angle(std::f64::consts::PI);  // Point left
        
        // Rotate all the way around (360°)
        for _ in 0..200 {
            gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 0.016);
        }
        
        // Should be pointing left (PI)
        assert!((gun.angle - std::f64::consts::PI).abs() < 0.1);
    }
    
    #[test]
    fn test_recoil_accumulation() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone());
        gun.add_recoil(0.1);
        
        assert!(gun.recoil_accumulated > 0.0);
        
        // Decay over time
        gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 0.5);
        assert!(gun.recoil_accumulated < 0.1);
    }
    
    #[test]
    fn test_autofire_spool_up() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone());
        
        // Start at initial cooldown (slow rate)
        assert_eq!(gun.autofire_cooldown_current, config.autofire_cooldown_start);
        
        // Spool up for 2 seconds
        gun.spool_up_autofire(2.0);
        
        // Should be at minimum
        assert_eq!(gun.autofire_cooldown_current, config.autofire_cooldown_min);
    }
    
    #[test]
    fn test_autofire_spool_down() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone());
        
        // Set to minimum
        gun.autofire_cooldown_current = config.autofire_cooldown_min;
        
        // Spool down for 2 seconds
        gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 2.0);
        
        // Should have increased
        assert!(gun.autofire_cooldown_current > config.autofire_cooldown_min);
    }
}