use rand::Rng;

use crate::config::GunConfig;

/// Gun mounted on player ship
#[derive(Debug, Clone)]
pub struct Gun {
    // Position relative to player center (anchored to ship vertices)
    pub offset_x: f64,
    pub offset_y: f64,
    
    // Current rotation (radians)
    pub angle: f64,
    
    // Base angle (natural pointing direction from ship center)
    pub base_angle: f64,
    
    // Dead zone offset (rotation applied to base angle)
    pub dead_zone_offset: f64,
    
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
    pub fn new(offset_x: f64, offset_y: f64, config: GunConfig, dead_zone_offset: f64) -> Self {
        // Calculate base angle: direction from ship center to gun position
        let base_angle = offset_y.atan2(offset_x);
        
        let initial_cooldown = config.autofire_cooldown_start;
        Gun {
            offset_x,
            offset_y,
            angle: base_angle,  // Start pointing in base direction
            base_angle,
            dead_zone_offset,
            target_angle: None,
            config,
            recoil_accumulated: 0.0,
            last_autofire_time: 0.0,
            autofire_cooldown_current: initial_cooldown,  // Start at slow rate
        }
    }
    
    /// Update gun tracking toward target with ship rotation
    pub fn update_tracking_with_ship(
        &mut self,
        player_facing: f64,          // Ship's facing direction
        player_rotation: f64,        // Ship's visual rotation (in world space)
        player_vx: f64,              // Player velocity X
        player_vy: f64,              // Player velocity Y
        player_speed: f64,             // Player speed
        movement_compensation: f64,     // Configurable compensation amount
        dt: f64
    ) {
        // Rotate base_angle with ship (the dead zone rotates with the ship)
        // base_angle is the direction from ship center to gun mount
        // We add player_rotation to rotate it into world space
        // Also apply dead_zone_offset to rotate the dead zone
        self.base_angle = self.offset_y.atan2(self.offset_x) + player_rotation + self.dead_zone_offset;
        
        // Fall through to standard tracking logic
        self.update_tracking(player_facing, player_vx, player_vy, player_speed, movement_compensation, dt);
    }
    
    /// Update gun tracking toward target
    pub fn update_tracking(
        &mut self, 
        player_angle: f64,           // Ship's facing direction
        player_vx: f64,              // Player velocity X
        player_vy: f64,              // Player velocity Y
        player_speed: f64,             // Player speed
        movement_compensation: f64,     // Configurable compensation amount
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
        
        // Calculate movement compensation
        // Maximum compensation when moving perpendicular to aim (90 degrees)
        // Zero compensation when moving parallel to aim (0 or 180 degrees)
        let aim_vx = target.cos();
        let aim_vy = target.sin();
        let speed = (player_vx * player_vx + player_vy * player_vy).sqrt();
        
        let compensated_target = if speed > 1.0 {
            // Normalize velocity
            let vx_norm = player_vx / speed;
            let vy_norm = player_vy / speed;
            
            // Dot product to get angle difference
            let parallelism = (aim_vx * vx_norm + aim_vy * vy_norm).abs();
            
            // Compensation is maximum when perpendicular (parallelism = 0)
            // Zero when parallel (parallelism = 1)
            let compensation_factor = (1.0 - parallelism) * (speed / player_speed.max(1.0));
            let compensation = compensation_factor * movement_compensation;
            
            // Apply compensation (lead the aim based on player movement)
            target + compensation * self.config.movement_compensation_scale
        } else {
            target
        };
        
        // Constrain to rotation limit around base_angle (dead zone points toward ship)
        // Calculate angle difference from base_angle
        let angle_diff = shortest_angle_diff(self.base_angle, compensated_target);
        
        // Normalize angle difference to [-π, π]
        let mut diff = angle_diff;
        while diff > std::f64::consts::PI {
            diff -= 2.0 * std::f64::consts::PI;
        }
        while diff < -std::f64::consts::PI {
            diff += 2.0 * std::f64::consts::PI;
        }
        
        // Constrain to rotation limit (convert degrees to radians)
        let limit_rad = (self.config.rotation_limit_degrees / 2.0).to_radians();  // ±half the limit
        let constrained_target = self.base_angle + diff.clamp(-limit_rad, limit_rad);
        
        // Smooth rotation toward constrained target
        let shortest_diff = shortest_angle_diff(self.angle, constrained_target);
        let max_rotation = self.config.rotation_speed * dt;
        
        if shortest_diff.abs() <= max_rotation {
            self.angle = constrained_target;
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
        // Increase from 0.5s to 0.1s over 1 second
        if self.autofire_cooldown_current > self.config.autofire_cooldown_min {
            self.autofire_cooldown_current -= self.config.autofire_spool_rate * dt;
            if self.autofire_cooldown_current < self.config.autofire_cooldown_min {
                self.autofire_cooldown_current = self.config.autofire_cooldown_min;
            }
        }
    }
    
    /// Set the target angle for this gun
    pub fn set_target_angle(&mut self, angle: f64) {
        self.target_angle = Some(angle);
    }
}

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
        let gun = Gun::new(-7.5, 10.0, config.clone(), 0.0);
        assert_eq!(gun.offset_x, -7.5);
        assert_eq!(gun.offset_y, 10.0);
        assert_eq!(gun.angle, -std::f64::consts::PI / 2.0);
    }
    
    #[test]
    fn test_gun_tracking() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone(), 0.0);
        gun.set_target_angle(0.0);  // Point right
        
        gun.update_tracking(0.0, 0.0, 0.0, 400.0, 1.0, 0.016);
        
        // Should rotate toward target
        assert!(gun.angle.abs() < 0.1);
    }
    
    #[test]
    fn test_recoil_accumulation() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone(), 0.0);
        gun.add_recoil(0.1);
        
        assert!(gun.recoil_accumulated > 0.0);
        
        // Decay over time
        gun.update_tracking(0.0, 0.0, 0.0, 400.0, 1.0, 0.5);
        assert!(gun.recoil_accumulated < 0.1);
    }
    
    #[test]
    fn test_autofire_spool_up() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone(), 0.0);
        
        // Start at initial cooldown (slow rate)
        assert_eq!(gun.autofire_cooldown_current, config.autofire_cooldown_start);
        
        // Spool up for 1 second
        gun.spool_up_autofire(1.0);
        
        // Should be at minimum
        assert_eq!(gun.autofire_cooldown_current, config.autofire_cooldown_min);
    }
    
    #[test]
    fn test_autofire_spool_down() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone(), 0.0);
        
        // Set to minimum
        gun.autofire_cooldown_current = config.autofire_cooldown_min;
        
        // Spool down for 1 second
        gun.update_tracking(0.0, 0.0, 0.0, 400.0, 1.0, 1.0);
        
        // Should have increased
        assert!(gun.autofire_cooldown_current > config.autofire_cooldown_min);
    }
}