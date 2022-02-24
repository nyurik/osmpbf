#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Bbox {
    pub left: i64,
    pub right: i64,
    pub top: i64,
    pub bottom: i64,
}

impl Default for Bbox {
    fn default() -> Self {
        Bbox {
            left: i64::MAX,
            right: i64::MIN,
            top: i64::MIN,
            bottom: i64::MAX,
        }
    }
}

impl Bbox {
    pub fn add_node(&mut self, lat: i64, lon: i64) {
        if lon < self.left {
            self.left = lon;
        }
        if lon > self.right {
            self.right = lon;
        }
        if lat > self.top {
            self.top = lat;
        }
        if lat < self.bottom {
            self.bottom = lat;
        }
    }

    pub fn add_node_list(&mut self, lats: &[i64], lons: &[i64]) {
        for &x in lons {
            if x < self.left {
                self.left = x;
            }
            if x > self.right {
                self.right = x;
            }
        }
        for &y in lats {
            if y > self.top {
                self.top = y;
            }
            if y < self.bottom {
                self.bottom = y;
            }
        }
    }

    pub fn add_bbox(&mut self, bbox: Bbox) {
        if bbox.left < self.left {
            self.left = bbox.left;
        }
        if bbox.right > self.right {
            self.right = bbox.right;
        }
        if bbox.top > self.top {
            self.top = bbox.top;
        }
        if bbox.bottom < self.bottom {
            self.bottom = bbox.bottom;
        }
    }

    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bbox() {
        let mut bbox = Bbox::default();
        assert!(bbox.is_empty());
        bbox.add_node(20, 10);
        assert!(!bbox.is_empty());
        assert_eq!(
            bbox,
            Bbox {
                left: 10,
                right: 10,
                top: 20,
                bottom: 20
            }
        );
        bbox.add_node(25, 12);
        assert_eq!(
            bbox,
            Bbox {
                left: 10,
                right: 12,
                top: 25,
                bottom: 20
            }
        );
        bbox.add_node_list(&[27, 23], &[14, 8]);
        assert_eq!(
            bbox,
            Bbox {
                left: 8,
                right: 14,
                top: 27,
                bottom: 20
            }
        );
    }
}
