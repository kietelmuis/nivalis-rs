// trait voor wgpu::Color
pub trait ColorExtensions {
    fn random(rng: &mut impl rand::Rng) -> Self;
    fn lerp(&self, other: &Self, t: f32) -> Self;
    fn is_near(&self, other: &Self, threshold: f32) -> bool;
}

// trait voor wgpu::Color
impl ColorExtensions for wgpu::Color {
    fn random(rng: &mut impl rand::Rng) -> Self {
        wgpu::Color {
            r: rng.random_range(0.0..1.0) as f64,
            g: rng.random_range(0.0..1.0) as f64,
            b: rng.random_range(0.0..1.0) as f64,
            a: 1.0,
        }
    }

    fn lerp(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0) as f64;
        wgpu::Color {
            r: self.r + (other.r - self.r) * t,
            g: self.g + (other.g - self.g) * t,
            b: self.b + (other.b - self.b) * t,
            a: self.a + (other.a - self.a) * t,
        }
    }

    fn is_near(&self, other: &Self, threshold: f32) -> bool {
        let threshold = threshold as f64;

        let diff_r = (self.r - other.r).abs();
        let diff_g = (self.g - other.g).abs();
        let diff_b = (self.b - other.b).abs();

        let avg_diff = (diff_r + diff_g + diff_b) / 3.0;
        diff_r < threshold
            && diff_g < threshold
            && diff_b < threshold
            && avg_diff < (threshold * 0.5)
    }
}
