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
    pub fn add_node(&mut self, x: i64, y: i64) {
        if x < self.left {
            self.left = x
        }
        if x > self.right {
            self.right = x
        }
        if y > self.top {
            self.top = y
        }
        if y < self.bottom {
            self.bottom = y
        }
    }

    pub fn add_node_list(&mut self, xs: &[i64], ys: &[i64]) {
        for &x in xs {
            if x < self.left {
                self.left = x
            }
            if x > self.right {
                self.right = x
            }
        }
        for &y in ys {
            if y > self.top {
                self.top = y
            }
            if y < self.bottom {
                self.bottom = y
            }
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
        bbox.add_node(10, 20);
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
        bbox.add_node(12, 25);
        assert_eq!(
            bbox,
            Bbox {
                left: 10,
                right: 12,
                top: 25,
                bottom: 20
            }
        );
        bbox.add_node_list(&[14, 8], &[27, 23]);
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
