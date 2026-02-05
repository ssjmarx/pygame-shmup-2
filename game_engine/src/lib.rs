use pyo3::prelude::*;

mod command;
mod config;
mod gun;
mod object_pool;
mod projectile;
mod star;
mod state;

use command::Command;
use config::{GunConfig, ProjectileConfig, PlayerGunConfig};
use gun::Gun;
use projectile::{Projectile, ProjectileRenderData};
use star::StarRenderData;
use state::GameState;

/// Python-accessible game engine wrapper
#[pyclass]
pub struct GameEngine {
    state: GameState,
}

#[pymethods]
impl GameEngine {
    /// Create new game engine instance
    #[new]
    fn new() -> Self {
        let state = GameState::new();
        GameEngine { state }
    }

    /// Send a movement command (direction only)
    fn send_command(&mut self, command_type: &str) {
        let cmd = match command_type {
            "move_up" => Command::MoveUp,
            "move_down" => Command::MoveDown,
            "move_left" => Command::MoveLeft,
            "move_right" => Command::MoveRight,
            _ => return,
        };
        self.state.add_command(cmd);
    }
    
    /// Set alt mode state
    fn set_alt_mode(&mut self, enabled: bool) {
        self.state.add_command(Command::ToggleAltMode(enabled));
    }
    
    /// Set boost mode state
    fn set_boost_mode(&mut self, enabled: bool) {
        self.state.add_command(Command::ToggleBoostMode(enabled));
    }
    
    /// Set control mode state
    fn set_control_mode(&mut self, enabled: bool) {
        self.state.add_command(Command::ToggleControlMode(enabled));
    }
    
    /// Set mouse target position
    fn set_mouse_target(&mut self, x: f64, y: f64) {
        // Get camera position from GameState (single source of truth)
        let cam_x = self.state.get_camera_x();
        let cam_y = self.state.get_camera_y();
        self.state.add_command(Command::SetMouseTarget(x, y, cam_x, cam_y));
    }
    
    /// Set target entity for tracking shots
    fn set_target_entity(&mut self, entity_id: Option<usize>) {
        self.state.add_command(Command::SetTargetEntity(entity_id));
    }
    
    /// Start shooting tracking bullets
    fn start_shooting_tracking(&mut self) {
        self.state.add_command(Command::StartShootingTracking);
    }
    
    /// Stop shooting tracking bullets
    fn stop_shooting_tracking(&mut self) {
        self.state.add_command(Command::StopShootingTracking);
    }
    
    /// Start autofiring
    fn start_autofire(&mut self) {
        self.state.add_command(Command::StartAutoFire);
    }
    
    /// Stop autofiring
    fn stop_autofire(&mut self) {
        self.state.add_command(Command::StopAutoFire);
    }

    /// Update game state by dt seconds
    fn update(&mut self, dt: f64) {
        self.state.update(dt);
    }

    /// Get render data for Python
    fn get_render_data(&self, py: Python) -> PyResult<PyObject> {
        let player = self.state.get_player();
        // Get camera position from GameState (single source of truth)
        let cam_x = self.state.get_camera_x();
        let cam_y = self.state.get_camera_y();
        let star_data = self.state.get_star_render_data();
        let projectile_data = self.state.get_projectile_render_data();
        let (left_gun_angle, right_gun_angle) = self.state.get_gun_angles();

        // Convert star data to Python list
        let stars_list = pyo3::types::PyList::new_bound(py, star_data.iter().map(|star| {
            let star_dict = pyo3::types::PyDict::new_bound(py);
            star_dict.set_item("x", star.x).unwrap();
            star_dict.set_item("y", star.y).unwrap();
            
            // Convert StarShape to string
            let shape_str = match star.shape {
                star::StarShape::Circle => "circle",
                star::StarShape::FourPoint => "four_point",
                star::StarShape::SixPoint => "six_point",
            };
            
            // Convert StarColor to string
            let color_str = match star.color {
                star::StarColor::White => "white",
                star::StarColor::LightBlue => "light_blue",
                star::StarColor::Cyan => "cyan",
                star::StarColor::LightPurple => "light_purple",
                star::StarColor::Pink => "pink",
                star::StarColor::PaleYellow => "pale_yellow",
            };
            
            star_dict.set_item("shape", shape_str).unwrap();
            star_dict.set_item("color", color_str).unwrap();
            star_dict.set_item("size", star.size).unwrap();
            star_dict.set_item("twinkle", star.twinkle).unwrap();
            star_dict.unbind()
        }));

        // Convert projectile data to Python list
        let projectiles_list = pyo3::types::PyList::new_bound(py, projectile_data.iter().map(|proj| {
            let proj_dict = pyo3::types::PyDict::new_bound(py);
            proj_dict.set_item("x", proj.x).unwrap();
            proj_dict.set_item("y", proj.y).unwrap();
            proj_dict.set_item("rotation", proj.rotation).unwrap();
            proj_dict.set_item("length", proj.length).unwrap();
            proj_dict.set_item("width", proj.width).unwrap();
            proj_dict.set_item("color", (proj.color.0, proj.color.1, proj.color.2)).unwrap();
            proj_dict.unbind()
        }));

        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("player_x", player.x)?;
        dict.set_item("player_y", player.y)?;
        dict.set_item("player_rotation", player.rotation)?;
        dict.set_item("camera_x", cam_x)?;
        dict.set_item("camera_y", cam_y)?;
        dict.set_item("player_vx", player.vx)?;
        dict.set_item("player_vy", player.vy)?;
        dict.set_item("stars", stars_list)?;
        dict.set_item("projectiles", projectiles_list)?;
        dict.set_item("left_gun_angle", left_gun_angle)?;
        dict.set_item("right_gun_angle", right_gun_angle)?;
        Ok(dict.unbind().into())
    }
}

/// Python module definition
#[pymodule]
fn game_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GameEngine>()?;
    Ok(())
}