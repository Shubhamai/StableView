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
    fn new(t0: f64, x0: f64, dx0: f64, min_cutoff: f64, beta: f64, d_cutoff: f64) -> Self {
        Self {
            min_cutoff,
            beta,
            d_cutoff,
            x_prev: x0,
            dx_prev: dx0,
            t_prev: t0,
        }
    }

    fn smoothing_factor(&self, t_e: f64, cutoff:f64) -> f64 {
        let r = 2.0 * std::f64::consts::PI * cutoff * t_e;
        r / (r + 1.0)
    }

    fn exponential_smoothing(&self, a: f64, x: f64, x_prev: f64) -> f64 {
        a * x + (1.0 - a) * x_prev
    }

    fn __call__(&mut self, t: f64, x: f64) -> f64 {
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

#[test]
fn test_euro_filter() {
    use std::f64;
    use std::f64::consts::PI;
    use gnuplot::{Figure, Caption, Color, Graph};

    // Create the filter with the initial values
    let mut filter = OneEuroFilter::new(0.0, 1., 0.0, 1.0, 0.0, 1.0);

    // Create vectors to store the original and filtered sin values
    let mut t_values:Vec<f64> = Vec::new();
    let mut x_values:Vec<f64> = Vec::new();
    let mut x_filtered_values:Vec<f64> = Vec::new();

    // Iterate over the sin values and apply the filter
    for i in 1..100 {
        // Compute the sin value at the current time
        let t = i as f64; // ! If t == 0, then zero division occurs at dx = (x - self.x_prev) / t_e, resulting in all values becoming None
        let x = (0.1*t).sin();

        // Filter the sin value
        let x_filtered = filter.__call__(t, x);

        // Print the original and filtered sin values
        println!("t = {:.2}, x = {:.2}, x_filtered = {:.2}", t, x, x_filtered);

        t_values.push(t);
        x_values.push(x);
        x_filtered_values.push(x_filtered);
    }

    let mut fg = Figure::new();

    // Add a 2D plot with the original and filtered sin values
    fg.axes2d()
        // .set_title("Sin values", &[])
        .set_legend(Graph(0.5), Graph(0.9), &[], &[])
        .lines(&t_values, &x_values, &[Caption("Original"), Color("red")])
        .lines(&t_values, &x_filtered_values, &[Caption("Filtered"), Color("blue")]);

    // Save the plot to a file
    fg.save_to_png("tests.filter_out.png", 800, 600).unwrap();


}