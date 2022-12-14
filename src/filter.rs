////////////////////////////////////////////// Rust Implementation of OneEuroFilter https://gery.casiez.net/1euro/ /////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
/////////////////////////////////////////////////// Created with help of OpenAI Chatbot ///////////////////////////////////////////////////////////////////
////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

use std::f64;

struct OneEuroFilter {
    min_cutoff: f64,
    beta: f64,
    d_cutoff: f64,
    x_prev: f64,
    dx_prev: f64,
    t_prev: f64,
}

impl OneEuroFilter {
    fn new(x0: f64, dx0: f64, min_cutoff: f64, beta: f64, d_cutoff: f64) -> Self {
        Self {
            min_cutoff,
            beta,
            d_cutoff,
            x_prev: x0,
            dx_prev: dx0,
            t_prev: 1.,
        }
    }

    fn smoothing_factor(&self, t_e: f64, cutoff: f64) -> f64 {
        let r = 2.0 * std::f64::consts::PI * cutoff * t_e;
        r / (r + 1.0)
    }

    fn exponential_smoothing(&self, a: f64, x: f64, x_prev: f64) -> f64 {
        a * x + (1.0 - a) * x_prev
    }

    fn __call__(&mut self, x: f64) -> f64 {
        let t = self.t_prev + 1.;

        let t_e = t - self.t_prev;

        let a_d = self.smoothing_factor(t_e, self.d_cutoff);
        let dx = (x - self.x_prev) / t_e;

        let dx_hat = self.exponential_smoothing(a_d, dx, self.dx_prev);

        let cutoff = self.min_cutoff + self.beta * dx_hat.abs();
        let a = self.smoothing_factor(t_e, cutoff);
        let x_hat = self.exponential_smoothing(a, x, self.x_prev);

        self.x_prev = x_hat;
        self.dx_prev = dx_hat;
        self.t_prev = t;

        x_hat
    }
}

pub struct EuroDataFilter {
    x_filter: OneEuroFilter,
    y_filter: OneEuroFilter,
    z_filter: OneEuroFilter,
    yaw_filter: OneEuroFilter,
    pitch_filter: OneEuroFilter,
    roll_filter: OneEuroFilter,
}

impl EuroDataFilter {
    pub fn new() -> Self {
        Self {
            x_filter: OneEuroFilter::new(0., 0., 0.0025, 0.01, 1.),
            y_filter: OneEuroFilter::new(0., 0., 0.0025, 0.01, 1.),
            z_filter: OneEuroFilter::new(0., 0., 0.0025, 0.01, 1.),
            yaw_filter: OneEuroFilter::new(0., 0., 0.0025, 0.01, 1.),
            pitch_filter: OneEuroFilter::new(0., 0., 0.0025, 0.01, 1.),
            roll_filter: OneEuroFilter::new(0., 0., 0.0025, 0.01, 1.),
        }
    }

    pub fn filter_data(&mut self, data: [f64; 6]) -> [f64; 6] {
        let mut filtered_data = [0.; 6];

        filtered_data[0] = self.x_filter.__call__(data[0]);
        filtered_data[1] = self.y_filter.__call__(data[1]);
        filtered_data[2] = self.z_filter.__call__(data[2]);
        filtered_data[3] = self.yaw_filter.__call__(data[3]);
        filtered_data[4] = self.pitch_filter.__call__(data[4]);
        filtered_data[5] = self.roll_filter.__call__(data[5]);

        filtered_data
    }
}

#[test]
fn test_euro_filter() {
    use rand::Rng;

    // Create the filter with the initial values
    let mut filter = OneEuroFilter::new(1., 0.0, 0.0001, 0.1, 1.0);

    // Iterate over the sin values and apply the filter
    for i in 1..100 {
        // Compute the sin value at the current time
        let t = i as f64; // ! If t == 0, then zero division occurs at dx = (x - self.x_prev) / t_e, resulting in all values becoming None
        let x = (0.1 * t).sin() + rand::thread_rng().gen_range(0..2) as f64 / 10.0;

        // Filter the sin value
        let x_filtered = filter.__call__(x);

        // Print the original and filtered sin values
        println!("t = {:.2}, x = {:.2}, x_filtered = {:.2}", t, x, x_filtered);
    }
}
