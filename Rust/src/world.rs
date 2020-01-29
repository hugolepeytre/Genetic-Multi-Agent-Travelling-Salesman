pub struct Customer {
    x: i64,
    y: i64,
    duration: i64,
    load: i64,
}

impl Customer {
    pub fn dist(&self, x: i64, y: i64) -> i64 {
        (((x - self.x)*(x - self.x) + (y - self.y)*(y - self.y)) as f64).sqrt() as i64
    }

    pub fn _dist_cust(&self, cust: &Customer) -> i64 {
        self.dist(cust.x, cust.y)
    }

    pub fn _dist_dep(&self, dep: &Depot) -> i64 {
        self.dist(dep.x, dep.y)
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }

    pub fn duration(&self) -> i64 {
        self.duration
    }

    pub fn load(&self) -> i64 {
        self.load
    }

    pub fn init(x: i64, y: i64, duration: i64, load: i64) -> Customer {
        Customer{x, y, duration, load}
    }
}

pub struct Depot {
    x: i64,
    y: i64,
    max_duration: i64,
    max_load: i64,
    vehicles: i64,
}

impl Depot {
    pub fn dist(&self, x: i64, y: i64) -> i64 {
        (((x - self.x)*(x - self.x) + (y - self.y)*(y - self.y)) as f64).sqrt() as i64
    }

    pub fn over_duration(&self, dur: i64) -> bool {
        self.max_duration != 0 && dur > self.max_duration
    }

    pub fn over_load(&self, load: i64) -> bool {
        load > self.max_load
    }

    pub fn _over_limits(&self, load: i64, dur: i64) -> bool {
        self.over_duration(dur) || self.over_load(load)
    }

    pub fn _dist_dep(&self, dep: &Depot) -> i64 {
        self.dist(dep.x, dep.y)
    }

    pub fn _dist_cust(&self, cust: &Customer) -> i64 {
        self.dist(cust.x, cust.y)
    }

    pub fn x(&self) -> i64 {
        self.x
    }

    pub fn y(&self) -> i64 {
        self.y
    }

    pub fn max_duration(&self) -> i64 {
        self.max_duration
    }

    pub fn max_load(&self) -> i64 {
        self.max_load
    }

    pub fn vehicles(&self) -> i64 {
        self.vehicles
    }

    pub fn init(x: i64, y: i64, max_duration: i64, max_load: i64, vehicles: i64) -> Depot {
        Depot{x, y, max_duration, max_load, vehicles}
    }
}