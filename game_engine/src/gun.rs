use crate::config::GunConfig;

/// Fire sector determines which gun(s) should fire
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FireSector {
    Left,
    Right,
    Both,
}

/// Determine which gun(s) should fire based on mouse position relative to player
pub fn get_fire_sector(mouse_angle: f64, player_facing: f64) -> FireSector {
    let relative_angle = normalize_angle(mouse_angle - player_facing);
    // Convert to degrees for easier sector comparison
    let deg = relative_angle.to_degrees();
    
    // Check overlap zones first (both guns fire)
    // Front overlap: -15° to +15°
    // Rear overlap: ±165° to ±195° (i.e., >= 165° or <= -165°)
    if deg.abs() <= 15.0 || deg.abs() >= 165.0 {
        FireSector::Both
    } else if deg > 0.0 {
        // Positive angle (left side of player)
        FireSector::Left
    } else {
        // Negative angle (right side of player)
        FireSector::Right
    }
}

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
    
    // Spool level (0.0 to 1.0, represents 0-100%)
    pub spool_level: f64,
    
    // Track if actively spooling up (to prevent spool down from cancelling)
    is_spooling: bool,
    
    // Rotational constraints
    base_angle: f64,              // Natural position (offset from ship center)
    arc_half_width: f64,          // 100° = half of 200° range
    
    // Calculated each frame based on player rotation
    arc_min: f64,                 // Minimum valid angle (world coords)
    arc_max: f64,                 // Maximum valid angle (world coords)
    dead_zone_center: f64,        // Center of forbidden zone
    
    // Configuration
    config: GunConfig,
}

impl Gun {
    pub fn new(offset_x: f64, offset_y: f64, config: GunConfig) -> Self {
        // Calculate base angle from ship center to gun (in screen coordinates)
        // atan2(y, x) gives angle where: 0=right, PI/2=up, PI=left, -PI/2=down
        // Screen coordinates use: 0=up, PI/2=right, PI=down, 3PI/2=left
        let base_angle_raw = offset_y.atan2(offset_x);
        let initial_cooldown = config.autofire_cooldown_start;
        let arc_half_width = std::f64::consts::PI * 100.0 / 180.0; // 100° in radians
        let quarter_pi = std::f64::consts::PI / 4.0; // 45° in radians
        
        // Adjust base_angle to fine-tune dead zones:
        // Right gun (negative x): rotate CCW by 45° (add +PI/4)
        // Left gun (positive x): rotate CW by 45° (add -PI/4)
        let base_angle = if offset_x < 0.0 {
            // Right gun - CCW 45°
            normalize_angle(base_angle_raw + quarter_pi)
        } else {
            // Left gun - CW 45°
            normalize_angle(base_angle_raw - quarter_pi)
        };
        
        Gun {
            offset_x,
            offset_y,
            angle: base_angle,  // Start pointing in base direction
            target_angle: None,
            config,
            recoil_accumulated: 0.0,
            last_autofire_time: 0.0,
            autofire_cooldown_current: initial_cooldown,
            spool_level: 0.0,  // Start at 0%
            is_spooling: false,
            
            // Rotational constraints
            base_angle,
            arc_half_width,
            arc_min: normalize_angle(base_angle - arc_half_width),
            arc_max: normalize_angle(base_angle + arc_half_width),
            dead_zone_center: normalize_angle(base_angle + std::f64::consts::PI),
        }
    }
    
    /// Update gun tracking toward target with ship rotation
    pub fn update_tracking_with_ship(
        &mut self,
        _player_facing: f64,
        player_rotation: f64,
        _player_vx: f64,
        _player_vy: f64,
        _player_speed: f64,
        _movement_compensation: f64,
        dt: f64
    ) {
        // Use player_rotation for arc calculations (matches visual rendering)
        // player_rotation includes +PI/2 offset for visual rendering
        // This ensures arc boundaries match where guns are actually positioned
        self.update_tracking(player_rotation, player_rotation, 0.0, 0.0, 1.0, dt);
    }
    
    /// Update gun tracking toward target
    pub fn update_tracking(
        &mut self, 
        player_facing: f64,
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
        
        // Spool down: decrease spool_level back toward 0%
        // Only spool down if NOT actively spooling up this frame
        if self.spool_level > 0.0 && !self.is_spooling {
            self.spool_level -= dt / self.config.spool_down_time;
            if self.spool_level < 0.0 {
                self.spool_level = 0.0;
            }
        }
        
        // Reset spooling flag for next frame
        self.is_spooling = false;
        
        // Update arc boundaries based on player rotation
        self.update_arc_boundaries(player_facing);
        
        // If no target, return
        let target = match self.target_angle {
            Some(angle) => angle,
            None => return,
        };
        
        // Use continuous validity check to rotate toward target while avoiding dead zone
        let max_rotation = self.config.rotation_speed * dt;
        self.rotate_toward_target_safely(target, max_rotation);
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
        // Calculate cooldown based on spool level
        // Spool 0% = longest cooldown (autofire_cooldown_start)
        // Spool 100% = shortest cooldown (autofire_cooldown_min)
        let range = self.config.autofire_cooldown_start - self.config.autofire_cooldown_min;
        let cooldown = self.config.autofire_cooldown_start - (range * self.spool_level);
        
        if current_time - self.last_autofire_time >= cooldown {
            self.last_autofire_time = current_time;
            return true;  // Ready to fire
        }
        false
    }
    
    /// Spool up autofire rate (call when holding mouse)
    pub fn spool_up_autofire(&mut self, dt: f64) {
        // Set flag to prevent spool down this frame
        self.is_spooling = true;
        
        // Increase spool_level toward 100% (1.0)
        if self.spool_level < 1.0 {
            self.spool_level += dt / self.config.spool_up_time;
            if self.spool_level > 1.0 {
                self.spool_level = 1.0;
            }
        }
    }
    
    /// Set target angle for this gun
    pub fn set_target_angle(&mut self, angle: f64) {
        self.target_angle = Some(angle);
    }
}

/// Normalize angle to [-π, π]
fn normalize_angle(angle: f64) -> f64 {
    let mut a = angle;
    while a > std::f64::consts::PI {
        a -= 2.0 * std::f64::consts::PI;
    }
    while a < -std::f64::consts::PI {
        a += 2.0 * std::f64::consts::PI;
    }
    a
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

/// Clamp a value between min and max
fn clamp(value: f64, min: f64, max: f64) -> f64 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

impl Gun {
    /// Update arc boundaries based on player rotation
    fn update_arc_boundaries(&mut self, player_facing: f64) {
        // Arc center is base_angle + player_facing
        // This ensures that gun's valid arc rotates with the ship
        let arc_center = normalize_angle(self.base_angle + player_facing);
        self.arc_min = normalize_angle(arc_center - self.arc_half_width);
        self.arc_max = normalize_angle(arc_center + self.arc_half_width);
        
        // Dead zone points from gun toward ship center
        // It's the angle pointing from gun's position toward (0,0)
        // For a gun at angle theta from center, direction toward center is theta + PI
        self.dead_zone_center = normalize_angle(self.base_angle + player_facing + std::f64::consts::PI);
    }
    
    /// Check if angle is within valid arc (handles wraparound)
    fn is_angle_valid(&self, angle: f64) -> bool {
        // Handle wraparound case (arc may cross 0°/360°)
        if self.arc_min < self.arc_max {
            // Normal case: no wraparound
            angle >= self.arc_min && angle <= self.arc_max
        } else {
            // Wraparound case: arc crosses 0°
            angle >= self.arc_min || angle <= self.arc_max
        }
    }
    
    /// Rotate gun to nearest valid arc boundary
    fn rotate_to_nearest_valid_edge(&mut self) {
        let dist_to_min = shortest_angle_diff(self.angle, self.arc_min);
        let dist_to_max = shortest_angle_diff(self.angle, self.arc_max);
        
        if dist_to_min.abs() < dist_to_max.abs() {
            self.angle = self.arc_min;
        } else {
            self.angle = self.arc_max;
        }
    }
    
    /// Ensure current angle is within valid arc
    fn clamp_to_valid_arc(&mut self) {
        if !self.is_angle_valid(self.angle) {
            self.rotate_to_nearest_valid_edge();
        }
    }
    
    /// Check if gun has reached or passed target angle
    fn angle_reached_target(&self, target: f64) -> bool {
        let diff = shortest_angle_diff(self.angle, target);
        diff.abs() < 0.01
    }
    
    /// Rotate toward target while avoiding dead zone
    fn rotate_toward_target_safely(&mut self, target: f64, max_step: f64) {
        // Try rotating directly toward target
        let direct_diff = shortest_angle_diff(self.angle, target);
        let candidate_angle = normalize_angle(self.angle + clamp(direct_diff, -max_step, max_step));
        
        // Check if this step is valid (not in dead zone)
        if self.is_angle_valid(candidate_angle) {
            // Direct rotation is safe
            self.angle = candidate_angle;
            
            // If we reached or passed the target but it's in dead zone,
            // stop at the nearest valid boundary
            if self.angle_reached_target(target) && !self.is_angle_valid(target) {
                self.rotate_to_nearest_valid_edge();
            }
        } else {
            // Direct rotation would cross dead zone - rotate the other way
            let opposite_direction = -direct_diff.signum();
            let opposite_step = opposite_direction * max_step;
            self.angle = normalize_angle(self.angle + opposite_step);
            
            // Clamp to valid arc if we're going the wrong way
            self.clamp_to_valid_arc();
        }
    }
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
        // Angle is base_angle which is atan2(10.0, -7.5)
        let expected = 10.0_f64.atan2(-7.5);
        assert!((gun.angle - expected).abs() < 0.01);
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
        
        // Gun should be at valid arc boundary (not PI which is in dead zone)
        assert!(gun.is_angle_valid(gun.angle));
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
        
        // Start at 0% spool
        assert_eq!(gun.spool_level, 0.0);
        
        // Spool up for 2 seconds (half of 4s spool time)
        gun.spool_up_autofire(2.0);
        
        // Should be at 50%
        assert!((gun.spool_level - 0.5).abs() < 0.01);
        
        // Spool up for another 2 seconds (total 4s)
        gun.spool_up_autofire(2.0);
        
        // Should be at 100%
        assert_eq!(gun.spool_level, 1.0);
    }
    
    #[test]
    fn test_autofire_spool_down() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone());
        
        // Start at 100% spool
        gun.spool_level = 1.0;
        
        // Spool down for 2 seconds (half of 4s spool time)
        gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 2.0);
        
        // Should be at 50%
        assert!((gun.spool_level - 0.5).abs() < 0.01);
        
        // Spool down for another 2 seconds
        gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 2.0);
        
        // Should be at 0%
        assert_eq!(gun.spool_level, 0.0);
    }
    
    #[test]
    fn test_autofire_cooldown_from_spool() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config.clone());
        
        // At 0% spool, cooldown should be 1.0s
        assert!((gun.update_autofire(0.0) == true));
        
        let dt_0 = gun.last_autofire_time;
        assert!((gun.update_autofire(dt_0 + config.autofire_cooldown_start - 0.01) == false));
        assert!((gun.update_autofire(dt_0 + config.autofire_cooldown_start) == true));
        
        // At 100% spool, cooldown should be 0.1s
        gun.spool_level = 1.0;
        gun.last_autofire_time = dt_0;
        assert!((gun.update_autofire(dt_0 + config.autofire_cooldown_min - 0.01) == false));
        assert!((gun.update_autofire(dt_0 + config.autofire_cooldown_min) == true));
    }
    
    // --- New tests for rotational constraints ---
    
    #[test]
    fn test_normalize_angle() {
        // Normalize angles to [-π, π]
        let a = normalize_angle(0.0);
        assert!((a - 0.0).abs() < 0.001);
        
        let a = normalize_angle(std::f64::consts::PI);
        assert!((a - std::f64::consts::PI).abs() < 0.001);
        
        let a = normalize_angle(-std::f64::consts::PI);
        assert!((a - (-std::f64::consts::PI)).abs() < 0.001);
        
        let a = normalize_angle(3.0 * std::f64::consts::PI);
        assert!((a - std::f64::consts::PI).abs() < 0.001);
        
        let a = normalize_angle(-3.0 * std::f64::consts::PI);
        assert!((a - (-std::f64::consts::PI)).abs() < 0.001);
    }
    
    #[test]
    fn test_shortest_angle_diff() {
        // Test shortest path
        let diff = shortest_angle_diff(0.0, 0.5);
        assert!((diff - 0.5).abs() < 0.001);
        
        // Test wraparound (go the other way)
        let diff = shortest_angle_diff(3.0, -3.0);
        assert!(diff.abs() < 1.0); // Should wrap, not go the long way
        
        // Test exact opposite
        let diff = shortest_angle_diff(0.0, std::f64::consts::PI);
        assert!((diff - std::f64::consts::PI).abs() < 0.001);
    }
    
    #[test]
    fn test_fire_sector_front_overlap() {
        // Front overlap zone: -15° to +15°
        let player_facing = 0.0;
        
        let sector = get_fire_sector(0.0, player_facing);
        assert_eq!(sector, FireSector::Both);
        
        let sector = get_fire_sector(0.2, player_facing); // ~11.5°
        assert_eq!(sector, FireSector::Both);
        
        let sector = get_fire_sector(-0.2, player_facing); // ~-11.5°
        assert_eq!(sector, FireSector::Both);
    }
    
    #[test]
    fn test_fire_sector_rear_overlap() {
        // Rear overlap zone: >= 165° or <= -165°
        let player_facing = 0.0;
        
        // 180° is in rear overlap
        let sector = get_fire_sector(std::f64::consts::PI, player_facing);
        assert_eq!(sector, FireSector::Both);
        
        // 170° is in rear overlap
        let sector = get_fire_sector(170.0_f64.to_radians(), player_facing);
        assert_eq!(sector, FireSector::Both);
        
        // -170° is in rear overlap
        let sector = get_fire_sector(-170.0_f64.to_radians(), player_facing);
        assert_eq!(sector, FireSector::Both);
        
        // 160° is NOT in rear overlap (it's in left sector)
        let sector = get_fire_sector(160.0_f64.to_radians(), player_facing);
        assert_eq!(sector, FireSector::Left);
        
        // -160° is NOT in rear overlap (it's in right sector)
        let sector = get_fire_sector(-160.0_f64.to_radians(), player_facing);
        assert_eq!(sector, FireSector::Right);
    }
    
    #[test]
    fn test_fire_sector_left() {
        // Left sector: +15° to +165°
        let player_facing = 0.0;
        
        let sector = get_fire_sector(1.0, player_facing); // ~57°
        assert_eq!(sector, FireSector::Left);
        
        let sector = get_fire_sector(2.0, player_facing); // ~115°
        assert_eq!(sector, FireSector::Left);
    }
    
    #[test]
    fn test_fire_sector_right() {
        // Right sector: -15° to -165°
        let player_facing = 0.0;
        
        let sector = get_fire_sector(-1.0, player_facing); // ~-57°
        assert_eq!(sector, FireSector::Right);
        
        let sector = get_fire_sector(-2.0, player_facing); // ~-115°
        assert_eq!(sector, FireSector::Right);
    }
    
    #[test]
    fn test_gun_arc_boundaries() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config);
        
        // Test initial boundaries
        assert!(gun.arc_min < 0.0);
        assert!(gun.arc_max > 0.0);
        assert!((gun.arc_max - gun.arc_min).abs() - 2.0 * gun.arc_half_width < 0.01);
    }
    
    #[test]
    fn test_is_angle_valid_normal_case() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config);
        gun.update_arc_boundaries(0.0);
        
        // Arc should be centered around base angle
        // Angles within arc should be valid
        assert!(gun.is_angle_valid(gun.arc_min));
        assert!(gun.is_angle_valid(gun.arc_max));
        assert!(gun.is_angle_valid((gun.arc_min + gun.arc_max) / 2.0));
        
        // Angles outside arc should be invalid
        assert!(!gun.is_angle_valid(gun.arc_min - 0.1));
        assert!(!gun.is_angle_valid(gun.arc_max + 0.1));
    }
    
    #[test]
    fn test_gun_dead_zone_avoidance() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config);
        
        // Set up gun at known position
        gun.angle = 0.0;
        gun.update_arc_boundaries(0.0);
        
        // Try to rotate into dead zone (opposite side)
        gun.set_target_angle(std::f64::consts::PI); // 180° opposite
        
        // Update for many frames
        for _ in 0..200 {
            gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 0.016);
        }
        
        // Gun should NOT be pointing at PI (dead zone)
        // It should be at one of the valid arc boundaries
        assert!(gun.is_angle_valid(gun.angle));
    }
    
    #[test]
    fn test_gun_valid_rotation() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config);
        
        // Set up gun
        gun.angle = 0.0;
        gun.update_arc_boundaries(0.0);
        
        // Target is within valid arc
        let valid_target = (gun.arc_min + gun.arc_max) / 2.0;
        gun.set_target_angle(valid_target);
        
        // Update
        gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 0.016);
        
        // Gun should rotate toward target
        let diff = shortest_angle_diff(gun.angle, valid_target);
        assert!(diff.abs() < 0.1); // Should have moved
    }
    
    #[test]
    fn test_gun_stays_in_valid_arc() {
        let config = GunConfig::default();
        let mut gun = Gun::new(0.0, 0.0, config);
        
        // Start at a valid angle
        gun.angle = 0.5;
        gun.update_arc_boundaries(0.0);
        
        // Try various targets, including some in dead zone
        for target in [1.0, 2.0, 3.0, -1.0, -2.0, std::f64::consts::PI] {
            gun.set_target_angle(target);
            
            // Update for several frames
            for _ in 0..50 {
                gun.update_tracking(0.0, 0.0, 0.0, 0.0, 1.0, 0.016);
            }
            
            // Gun should always be in valid arc
            assert!(gun.is_angle_valid(gun.angle), 
                "Gun angle {} is not in valid arc [{} to {}]", 
                gun.angle, gun.arc_min, gun.arc_max);
        }
    }
    
    #[test]
    fn test_clamp() {
        assert_eq!(clamp(0.5, 0.0, 1.0), 0.5);
        assert_eq!(clamp(-0.5, 0.0, 1.0), 0.0);
        assert_eq!(clamp(1.5, 0.0, 1.0), 1.0);
    }
}
