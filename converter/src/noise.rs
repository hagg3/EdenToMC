pub struct Noise2D {
    seed: u32,
}

impl Noise2D {
    pub fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn smoothstep(t: f64) -> f64 {
        t * t * (3.0 - 2.0 * t)
    }

    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + (b - a) * t
    }

    fn lattice(&self, x: i32, z: i32) -> f64 {
        let mut h = (x as u32).wrapping_mul(374761393);
        h ^= (z as u32).wrapping_mul(668265263);
        h ^= self.seed.wrapping_mul(2246822519);
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        h ^= h >> 16;
        ((h & 0xFFFFFF) as f64 / 0x7FFFFF as f64) - 1.0
    }

    fn value_noise(&self, x: f64, z: f64) -> f64 {
        let x0 = x.floor() as i32;
        let z0 = z.floor() as i32;
        let tx = Self::smoothstep(x - x0 as f64);
        let tz = Self::smoothstep(z - z0 as f64);
        let v00 = self.lattice(x0, z0);
        let v10 = self.lattice(x0 + 1, z0);
        let v01 = self.lattice(x0, z0 + 1);
        let v11 = self.lattice(x0 + 1, z0 + 1);
        let vx0 = Self::lerp(v00, v10, tx);
        let vx1 = Self::lerp(v01, v11, tx);
        Self::lerp(vx0, vx1, tz)
    }

    fn lattice3d(&self, x: i32, y: i32, z: i32) -> f64 {
        let mut h = (x as u32).wrapping_mul(374761393);
        h ^= (y as u32).wrapping_mul(3266489917);
        h ^= (z as u32).wrapping_mul(668265263);
        h ^= self.seed.wrapping_mul(2246822519);
        h = (h ^ (h >> 13)).wrapping_mul(1274126177);
        h ^= h >> 16;
        ((h & 0xFFFFFF) as f64 / 0x7FFFFF as f64) - 1.0
    }

    fn value_noise3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let x0 = x.floor() as i32;
        let y0 = y.floor() as i32;
        let z0 = z.floor() as i32;
        let tx = Self::smoothstep(x - x0 as f64);
        let ty = Self::smoothstep(y - y0 as f64);
        let tz = Self::smoothstep(z - z0 as f64);
        let c000 = self.lattice3d(x0, y0, z0);
        let c100 = self.lattice3d(x0 + 1, y0, z0);
        let c010 = self.lattice3d(x0, y0 + 1, z0);
        let c110 = self.lattice3d(x0 + 1, y0 + 1, z0);
        let c001 = self.lattice3d(x0, y0, z0 + 1);
        let c101 = self.lattice3d(x0 + 1, y0, z0 + 1);
        let c011 = self.lattice3d(x0, y0 + 1, z0 + 1);
        let c111 = self.lattice3d(x0 + 1, y0 + 1, z0 + 1);
        let x00 = Self::lerp(c000, c100, tx);
        let x10 = Self::lerp(c010, c110, tx);
        let x01 = Self::lerp(c001, c101, tx);
        let x11 = Self::lerp(c011, c111, tx);
        let y0v = Self::lerp(x00, x10, ty);
        let y1v = Self::lerp(x01, x11, ty);
        Self::lerp(y0v, y1v, tz)
    }

    pub fn fractal(&self, x: f64, z: f64, octaves: u32, base_freq: f64, persistence: f64) -> f64 {
        let mut amp = 1.0f64;
        let mut freq = base_freq;
        let mut total = 0.0f64;
        let mut norm = 0.0f64;
        for _ in 0..octaves {
            total += self.value_noise(x * freq, z * freq) * amp;
            norm += amp;
            amp *= persistence;
            freq *= 2.0;
        }
        if norm <= 0.0 { 0.0 } else { total / norm }
    }

    pub fn fractal3d(&self, x: f64, y: f64, z: f64, octaves: u32, base_freq: f64, persistence: f64) -> f64 {
        let mut amp = 1.0f64;
        let mut freq = base_freq;
        let mut total = 0.0f64;
        let mut norm = 0.0f64;
        for _ in 0..octaves {
            total += self.value_noise3d(x * freq, y * freq, z * freq) * amp;
            norm += amp;
            amp *= persistence;
            freq *= 2.0;
        }
        if norm <= 0.0 { 0.0 } else { total / norm }
    }
}
