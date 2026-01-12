//! 3D Camera
//!
//! Provides orbital, pan, and fly-through camera controls for 3D viewing.

use nalgebra as na;

/// 3D point type
pub type Point3 = na::Point3<f64>;

/// 3D vector type
pub type Vector3 = na::Vector3<f64>;

/// 3D camera uniform data
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera3DUniform {
    /// View-projection matrix
    pub view_proj: [[f32; 4]; 4],
    /// Camera position (for lighting calculations)
    pub camera_pos: [f32; 4],
}

impl Camera3DUniform {
    pub fn new() -> Self {
        Self {
            view_proj: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
            camera_pos: [0.0, 0.0, 10.0, 1.0],
        }
    }
}

impl Default for Camera3DUniform {
    fn default() -> Self {
        Self::new()
    }
}

/// 3D Camera with orbital controls
#[derive(Debug, Clone)]
pub struct Camera3D {
    /// Camera position (eye)
    pub position: Point3,

    /// Target point (look at)
    pub target: Point3,

    /// Up vector
    pub up: Vector3,

    /// Field of view (radians)
    pub fov: f64,

    /// Near clipping plane
    pub near: f64,

    /// Far clipping plane
    pub far: f64,

    /// Viewport width
    pub viewport_width: u32,

    /// Viewport height
    pub viewport_height: u32,

    /// Orbit distance
    orbit_distance: f64,

    /// Orbit yaw (horizontal angle in radians)
    orbit_yaw: f64,

    /// Orbit pitch (vertical angle in radians)
    orbit_pitch: f64,
}

impl Camera3D {
    /// Create a new 3D camera
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        let mut camera = Self {
            position: Point3::new(0.0, -100.0, 50.0),
            target: Point3::origin(),
            up: Vector3::z(),
            fov: 45.0_f64.to_radians(),
            near: 0.1,
            far: 10000.0,
            viewport_width,
            viewport_height,
            orbit_distance: 100.0,
            orbit_yaw: 0.0,
            orbit_pitch: 30.0_f64.to_radians(),
        };
        camera.update_position_from_orbit();
        camera
    }

    /// Update viewport size
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// Get the aspect ratio
    pub fn aspect_ratio(&self) -> f64 {
        self.viewport_width as f64 / self.viewport_height as f64
    }

    /// Get the view matrix
    pub fn view_matrix(&self) -> na::Matrix4<f64> {
        na::Matrix4::look_at_rh(&self.position, &self.target, &self.up)
    }

    /// Get the projection matrix (perspective)
    pub fn projection_matrix(&self) -> na::Matrix4<f64> {
        na::Matrix4::new_perspective(self.aspect_ratio(), self.fov, self.near, self.far)
    }

    /// Get the combined view-projection matrix
    pub fn view_projection_matrix(&self) -> na::Matrix4<f64> {
        self.projection_matrix() * self.view_matrix()
    }

    /// Get the view-projection as a f32 array for GPU
    pub fn view_projection_f32(&self) -> [[f32; 4]; 4] {
        let m = self.view_projection_matrix();
        [
            [m[(0, 0)] as f32, m[(1, 0)] as f32, m[(2, 0)] as f32, m[(3, 0)] as f32],
            [m[(0, 1)] as f32, m[(1, 1)] as f32, m[(2, 1)] as f32, m[(3, 1)] as f32],
            [m[(0, 2)] as f32, m[(1, 2)] as f32, m[(2, 2)] as f32, m[(3, 2)] as f32],
            [m[(0, 3)] as f32, m[(1, 3)] as f32, m[(2, 3)] as f32, m[(3, 3)] as f32],
        ]
    }

    /// Get the uniform data for GPU
    pub fn to_uniform(&self) -> Camera3DUniform {
        Camera3DUniform {
            view_proj: self.view_projection_f32(),
            camera_pos: [
                self.position.x as f32,
                self.position.y as f32,
                self.position.z as f32,
                1.0,
            ],
        }
    }

    // ========== Orbit Controls ==========

    /// Orbit the camera around the target
    pub fn orbit(&mut self, delta_yaw: f64, delta_pitch: f64) {
        self.orbit_yaw += delta_yaw;
        self.orbit_pitch = (self.orbit_pitch + delta_pitch).clamp(
            -89.0_f64.to_radians(),
            89.0_f64.to_radians(),
        );
        self.update_position_from_orbit();
    }

    /// Zoom by changing orbit distance
    pub fn zoom(&mut self, factor: f64) {
        self.orbit_distance = (self.orbit_distance * factor).clamp(1.0, 10000.0);
        self.update_position_from_orbit();
    }

    /// Zoom to a specific distance
    pub fn set_orbit_distance(&mut self, distance: f64) {
        self.orbit_distance = distance.clamp(1.0, 10000.0);
        self.update_position_from_orbit();
    }

    /// Pan the camera (move target and position together)
    pub fn pan(&mut self, delta_x: f64, delta_y: f64) {
        // Get camera-relative axes
        let forward = (self.target - self.position).normalize();
        let right = forward.cross(&self.up).normalize();
        let up = right.cross(&forward);

        // Scale by distance for consistent feel
        let scale = self.orbit_distance * 0.001;
        let offset = right * delta_x * scale + up * delta_y * scale;

        self.target += offset;
        self.update_position_from_orbit();
    }

    /// Update camera position from orbit parameters
    fn update_position_from_orbit(&mut self) {
        let x = self.orbit_distance * self.orbit_pitch.cos() * self.orbit_yaw.sin();
        let y = self.orbit_distance * self.orbit_pitch.cos() * self.orbit_yaw.cos();
        let z = self.orbit_distance * self.orbit_pitch.sin();

        self.position = self.target + Vector3::new(x, -y, z);
    }

    /// Look at a specific target
    pub fn look_at(&mut self, target: Point3) {
        self.target = target;
        self.orbit_distance = (self.position - target).norm();
        let dir = target - self.position;
        self.orbit_yaw = dir.y.atan2(dir.x);
        self.orbit_pitch = (dir.z / self.orbit_distance).asin();
    }

    /// Reset to default view
    pub fn reset(&mut self) {
        self.target = Point3::origin();
        self.orbit_distance = 100.0;
        self.orbit_yaw = 0.0;
        self.orbit_pitch = 30.0_f64.to_radians();
        self.update_position_from_orbit();
    }

    // ========== Standard Views ==========

    /// View from the top (Z+)
    pub fn view_top(&mut self) {
        self.orbit_yaw = 0.0;
        self.orbit_pitch = 89.0_f64.to_radians();
        self.update_position_from_orbit();
    }

    /// View from the bottom (Z-)
    pub fn view_bottom(&mut self) {
        self.orbit_yaw = 0.0;
        self.orbit_pitch = -89.0_f64.to_radians();
        self.update_position_from_orbit();
    }

    /// View from the front (Y-)
    pub fn view_front(&mut self) {
        self.orbit_yaw = 0.0;
        self.orbit_pitch = 0.0;
        self.update_position_from_orbit();
    }

    /// View from the back (Y+)
    pub fn view_back(&mut self) {
        self.orbit_yaw = std::f64::consts::PI;
        self.orbit_pitch = 0.0;
        self.update_position_from_orbit();
    }

    /// View from the left (X-)
    pub fn view_left(&mut self) {
        self.orbit_yaw = -std::f64::consts::FRAC_PI_2;
        self.orbit_pitch = 0.0;
        self.update_position_from_orbit();
    }

    /// View from the right (X+)
    pub fn view_right(&mut self) {
        self.orbit_yaw = std::f64::consts::FRAC_PI_2;
        self.orbit_pitch = 0.0;
        self.update_position_from_orbit();
    }

    /// Isometric view (common in CAD)
    pub fn view_isometric(&mut self) {
        self.orbit_yaw = 45.0_f64.to_radians();
        self.orbit_pitch = 35.264_f64.to_radians(); // atan(1/sqrt(2))
        self.update_position_from_orbit();
    }

    // ========== Fit to View ==========

    /// Fit a bounding box into view
    pub fn fit_to_box(&mut self, min: Point3, max: Point3, padding: f64) {
        let center = Point3::new(
            (min.x + max.x) / 2.0,
            (min.y + max.y) / 2.0,
            (min.z + max.z) / 2.0,
        );

        let diagonal = (max - min).norm();
        let distance = (diagonal / 2.0 + padding) / (self.fov / 2.0).tan();

        self.target = center;
        self.orbit_distance = distance.max(1.0);
        self.update_position_from_orbit();
    }

    /// Get the camera direction (normalized)
    pub fn direction(&self) -> Vector3 {
        (self.target - self.position).normalize()
    }

    /// Get the right vector
    pub fn right(&self) -> Vector3 {
        self.direction().cross(&self.up).normalize()
    }
}

impl Default for Camera3D {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_orbit() {
        let mut camera = Camera3D::new(800, 600);
        let initial_pos = camera.position;

        camera.orbit(0.1, 0.1);

        assert_ne!(camera.position, initial_pos);
    }

    #[test]
    fn test_camera_zoom() {
        let mut camera = Camera3D::new(800, 600);
        let initial_distance = camera.orbit_distance;

        camera.zoom(0.5);

        assert!((camera.orbit_distance - initial_distance * 0.5).abs() < 0.01);
    }

    #[test]
    fn test_standard_views() {
        let mut camera = Camera3D::new(800, 600);

        camera.view_top();
        assert!(camera.position.z > camera.target.z);

        camera.view_front();
        assert!(camera.position.y < camera.target.y);
    }
}
