use pyo3::prelude::*;

mod camera;
mod command;
mod object_pool;
mod star;
mod state;

use camera::Camera;
use command::Command;
use star::StarRenderData;
use state::GameState;

/// Python-accessible game engine wrapper
#[pyclass]
pub struct GameEngine {
    state: GameState,
    camera: Camera,
}

#[pymethods]
impl GameEngine {
    /// Create new game engine instance
    #[new]
    fn new() -> Self {
        let state = GameState::new();
        let camera = Camera::new();
        GameEngine { state, camera }
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
        let (cam_x, cam_y) = self.camera.get_offset();
        self.state.add_command(Command::SetMouseTarget(x, y, cam_x, cam_y));
    }

    /// Update game state by dt seconds
    fn update(&mut self, dt: f64) {
        self.state.update(dt);
        self.camera.track_player(&self.state.get_player());
    }

    /// Get render data for Python
    fn get_render_data(&self, py: Python) -> PyResult<PyObject> {
        let player = self.state.get_player();
        let (cam_x, cam_y) = self.camera.get_offset();
        let star_data = self.state.get_star_render_data();

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

        let dict = pyo3::types::PyDict::new_bound(py);
        dict.set_item("player_x", player.x)?;
        dict.set_item("player_y", player.y)?;
        dict.set_item("player_rotation", player.rotation)?;
        dict.set_item("camera_x", cam_x)?;
        dict.set_item("camera_y", cam_y)?;
        dict.set_item("player_vx", player.vx)?;
        dict.set_item("player_vy", player.vy)?;
        dict.set_item("stars", stars_list)?;
        Ok(dict.unbind().into())
    }
}

/// Python module definition
#[pymodule]
fn game_engine(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GameEngine>()?;
    Ok(())
}