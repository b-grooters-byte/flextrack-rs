#[derive(Debug, Clone)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    pub fn distance(&self, p: &Point) -> f32 {
        f32::sqrt((self.x - p.x) * (self.x - p.x) + (self.y - p.y) * (self.y - p.y))
    }

    /// Gets the slope of a line segment defined by the endpoints self and p
    ///
    /// Gets the slope for 2 points where:
    ///      m =  cy / cx
    /// or:
    ///          (y2 - y1)
    ///      m = ---------
    ///          (x2 - x1)
    /// the point passed in the parameter list is considered as x2, y2
    pub fn slope(&self, p: &Point) -> f32 {
        let cy = p.y - self.y;
        let cx = p.x - self.x;
        if cx == 0.0 {
            return f32::NAN;
        }
        cy / cx
    }
}

impl Rect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Rect {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.x && p.y <= self.x + self.width && p.y >= self.y && p.y <= self.y + self.height
    }
}

pub struct Line {
    pub start: Point,
    pub end: Point,
}

pub struct Polygon {
    pub points: Vec<Point>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_contains() {
        let r = Rect::new(10.0, 10.0, 10.0, 10.0);
        assert!(r.contains(Point { x: 15.0, y: 15.0 }));
        assert!(!r.contains(Point { x: 5.0, y: 5.0 }));
    }

    #[test]
    fn test_slope() {
        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 1.0, y: 1.0 };
        assert_eq!(p1.slope(&p2), 1.0);

        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 1.0, y: 0.0 };
        assert_eq!(p1.slope(&p2), 0.0);

        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 0.0, y: 1.0 };
        assert!(p1.slope(&p2).is_nan());

        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 1.0, y: 2.0 };
        assert_eq!(p1.slope(&p2), 2.0);
    }

    #[test]
    fn test_distance() {
        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: 3.0, y: 4.0 };
        assert_eq!(p1.distance(&p2), 5.0);

        let p1 = Point { x: 0.0, y: 0.0 };
        let p2 = Point { x: -10.0, y: -10.0 };
        assert_eq!(p1.distance(&p2).floor(), 14.0);
    }
}
