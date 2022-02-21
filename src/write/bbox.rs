#[derive(Debug, Clone)]
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
            top: i64::MAX,
            bottom: i64::MIN,
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
        if y < self.top {
            self.top = y
        }
        if y > self.bottom {
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
            if y < self.top {
                self.top = y
            }
            if y > self.bottom {
                self.bottom = y
            }
        }
    }
}
