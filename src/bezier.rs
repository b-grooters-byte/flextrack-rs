
pub struct Bezier {
    resolution: f32,
    size: u16,
    modified: bool,
}

impl Bezier {
    /// Creates a new bezier curve with the specified parameters
    /// resolution - a value representing the resolution of the curve. 
    /// 0.0 < value <= 1.0. A smaller value is a higher resolution
    pub fn new(resolution: f32) -> Self {
        Bezier{
            resolution,
            size: 0,
            modified: false,
        }
    }

    pub fn set_resolution(&mut self, resolution: f32) {
        if self.resolution != resolution {
            self.resolution = resolution;
            self.modified = true;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new() {
        let b = Bezier::new(0.25);
        assert_eq!(0.25, b.resolution);
    }

    #[test]
    fn test_set_resolution() {
        let mut b = Bezier::new(0.25);
        assert!(!b.modified);
        b.set_resolution(0.15);
        assert!(b.modified);
        assert_eq!(0.15, b.resolution);
    }
}