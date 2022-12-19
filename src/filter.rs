/// Rust Implementation of OneEuroFilter https://gery.casiez.net/1euro/ to filter real-time noisy signals
/// Visit the site to learn more about the parameters involved and how to tune them
/// The pseudocode is originajlly from https://github.com/jaantollander/OneEuroFilter, which is further modified for our use case  
use std::f32;

// ! Need Default values
struct OneEuroFilter {
    // Parameters
    min_cutoff: f32,
    beta: f32,
    d_cutoff: f32,

    // Previous Values
    x_prev: f32,
    dx_prev: f32,
}

// TODO : A way to have default value in filter
impl OneEuroFilter {
    fn new(x0: f32, dx0: f32, min_cutoff: f32, beta: f32, d_cutoff: f32) -> Self {
        Self {
            min_cutoff,
            beta,
            d_cutoff,

            x_prev: x0,
            dx_prev: dx0,
        }
    }

    fn smoothing_factor(&self, t_e: f32, cutoff: f32) -> f32 {
        let r = 2.0 * std::f32::consts::PI * cutoff * t_e;
        r / (r + 1.0)
    }

    fn exponential_smoothing(&self, a: f32, x: f32, x_prev: f32) -> f32 {
        a.mul_add(x, (1.0 - a) * x_prev)
    }

    fn run(&mut self, x: f32) -> f32 {
        let t_e = 1.; // constant change in time

        let a_d = self.smoothing_factor(t_e, self.d_cutoff);
        let dx = (x - self.x_prev) / t_e;

        self.dx_prev = self.exponential_smoothing(a_d, dx, self.dx_prev);

        let cutoff = self.beta.mul_add(self.dx_prev.abs(), self.min_cutoff);
        let a = self.smoothing_factor(t_e, cutoff);
        self.x_prev = self.exponential_smoothing(a, x, self.x_prev);

        self.x_prev
    }
}

// TODO : Need to clean this up
pub struct EuroDataFilter {
    x: OneEuroFilter,
    y: OneEuroFilter,
    z: OneEuroFilter,
    yaw: OneEuroFilter,
    pitch: OneEuroFilter,
    roll: OneEuroFilter,
}

impl EuroDataFilter {
    pub fn new(min_cutoff: f32, beta: f32) -> Self {
        Self {
            x: OneEuroFilter::new(0., 0., min_cutoff, beta, 1.),
            y: OneEuroFilter::new(0., 0., min_cutoff, beta, 1.),
            z: OneEuroFilter::new(0., 0., min_cutoff, beta, 1.),
            yaw: OneEuroFilter::new(0., 0., min_cutoff, beta, 1.),
            pitch: OneEuroFilter::new(0., 0., min_cutoff, beta, 1.),
            roll: OneEuroFilter::new(0., 0., min_cutoff, beta, 1.),
        }
    }

    pub fn filter_data(&mut self, data: [f32; 6]) -> [f32; 6] {
        let mut filtered_data = [0.; 6];

        filtered_data[0] = self.x.run(data[0]);
        filtered_data[1] = self.y.run(data[1]);
        filtered_data[2] = self.z.run(data[2]);
        filtered_data[3] = self.yaw.run(data[3]);
        filtered_data[4] = self.pitch.run(data[4]);
        filtered_data[5] = self.roll.run(data[5]);

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
        // Compute the noisy sin value
        let x = (0.1 * i as f32).sin();
        let x_noisy = x + (rand::thread_rng().gen_range(0..10) as f32 / 10.0);

        // Filter the noisy sin value
        let x_filtered = filter.run(x_noisy);

        // Print the original and filtered sin values
        println!(
            "x {:.2}, noisy {:.2}, filtered {:.2}",
            x, x_noisy, x_filtered
        );
    }
}
