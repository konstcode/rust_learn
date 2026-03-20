/*
* ● Chapter 13 Task: Data Pipeline

* Build a small statistics processor using closures and iterators. Create a file ch13_task/src/main.rs:
* Implement the following:

* 1. A function apply_pipeline that takes a Vec<f64> and a Vec of transformation closures
* (Vec<Box<dyn Fn(f64) -> f64>>), and returns a new Vec<f64> where each element has
* passed through all transformations in order.
* 2. A function summarize that takes a &[f64] and returns a tuple (f64, f64, f64) — (min, max, average)
*    — implemented entirely with iterator methods (no manual loops).
* 3. In main, demonstrate by:
*   - Starting with: vec![1.0, -3.5, 4.2, -2.1, 7.8, 0.5, -1.0, 6.3]
*   - Building a pipeline of 3 closures: absolute value → square → add 1.0
*   - Running apply_pipeline
*   - Running summarize on the result
*   - Printing the transformed values and the summary

* Constraints:
* - No for loops anywhere — use only iterator methods
* - apply_pipeline must use fold or chained map to apply the closures
* - summarize must use iterator consumers (fold, min/max with floats, or a single fold that tracks all three)

* Hint: f64 doesn't implement Ord, so .min() / .max() won't work directly — you'll need fold or f64::min / f64::max.
*
*/

fn apply_pipeline(vec: Vec<f64>, funcs: Vec<Box<dyn Fn(f64) -> f64>>) -> Vec<f64> {
    vec.iter()
        .map(|e| funcs.iter().fold(*e, |result, f| f(result)))
        .collect()
}

// return min, max, average
fn summarize(data: &[f64]) -> (f64, f64, f64) {
    (
        data.iter().cloned().reduce(f64::min).unwrap_or(0.),
        data.iter().cloned().reduce(f64::max).unwrap_or(0.),
        data.iter().sum::<f64>() / data.len() as f64,
    )
}

fn main() {
    let vec = vec![1.0, -3.5, 4.2, -2.1, 7.8, 0.5, -1.0, 6.3];

    let funcs: Vec<Box<dyn Fn(f64) -> f64>> = vec![
        Box::new(|x| x.abs()),
        Box::new(|x| x * x),
        Box::new(|x| x + 1.0),
    ];

    let updated_vec = apply_pipeline(vec, funcs);
    println!("{:#?}", &updated_vec);
    println!("{:#?}", summarize(updated_vec.as_slice()));
}
