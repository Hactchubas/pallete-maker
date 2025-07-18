use crate::colors;
use crate::weights;

use colors::LAB;
use rand::{Rng, distributions::WeightedIndex, prelude::*};
use std::{cmp, thread};
use weights::WeightFn;

pub type Pixels = Vec<LAB>;

fn find_clusters(pixels: &Pixels, means: &Pixels, num_colors: &usize) -> Vec<Pixels> {
    let num_threads: usize = if *num_colors <= 10 { *num_colors } else { 5 };

    let num_pixels = pixels.len();
    let sample_size = num_pixels / num_threads;
    let mut clusters: Vec<Pixels> = vec![Vec::new(); *num_colors];

    thread::scope(|s| {
        let mut threads = vec![];

        // Data parallelism whrer thread operates on a sample of data
        for i in 0..num_threads {
            let start = i * sample_size;
            let end = cmp::min(start + sample_size, num_pixels);

            // Each thread is responsible in finding the nearest cluster mean for each point in the
            // sample (map phase)

            threads.push(s.spawn(move || {
                let mut clusters: Vec<Vec<LAB>> = vec![Vec::new(); *num_colors];
                for pixel_idx in start..end {
                    let color = &pixels[pixel_idx];
                    clusters[color.nearest(&means).0].push(color.clone());
                }
                return clusters;
            }))
        }

        // Results from each thread is combined (or reduced)
        for t in threads {
            let mut mid_clusters = t.join().unwrap();
            for (i, cluster) in mid_clusters.iter_mut().enumerate() {
                clusters[i].append(cluster);
            }
        }
    });

    clusters
}

// Parallelized num_colors-means++ clustering to create the pallete from pixels
pub fn pallete(pixels: &Pixels, weight: WeightFn, num_colors: &usize) -> Vec<(LAB, f32)> {
    // Values referenced from
    // https://scikit-learn.org/stable/modules/generated/sklearn.cluster.Kmenas.html
    const TOLERANCE: f32 = 1e-4;
    const MAX_ITER: u16 = 300;

    let mut rng = rand::thread_rng();

    // Randomly pick the starting cluster center
    let i: usize = rng.gen_range(0..pixels.len());
    let mut means: Pixels = vec![pixels[i].clone()];

    // Pick the remaining (num_colors-1) means
    for _ in 0..(num_colors - 1) {
        // Calculate the (nearest_distance)² for every color in the image
        let distances: Vec<f32> = pixels
            .iter()
            .map(|color| (color.nearest(&means).1).powi(2))
            .collect();

        // Create a weighted distribuition based on distance²
        // If error occurs, return the means already found
        let dist = match WeightedIndex::new(&distances) {
            Ok(t) => t,
            Err(_) => {
                println!("Error in weighted distribuition");
                // Calculate the dominace of each color
                let mut pallete: Vec<(LAB, f32)> = means.iter().map(|c| (c.clone(), 0.0)).collect();

                let len = pixels.len() as f32;
                for color in pixels.iter() {
                    let near = color.nearest(&means).0;
                    pallete[near].1 += 1.0 / len;
                }
                return pallete;
            }
        };
        // Using the distances² as weights, pick a color and use it as a cluster center
        means.push(pixels[dist.sample(&mut rng)].clone());
    }

    let mut clusters: Vec<Vec<LAB>>;
    let mut iters_left = MAX_ITER;

    loop {
        // Assignment step: Clusters are formed in current iteration
        clusters = find_clusters(pixels, &means, num_colors);

        // Updation step: New cluster means are calculated
        let mut changed: bool = false;
        for i in 0..clusters.len() {
            let new_mean = recal_means(&clusters[i], weight);
            if means[i].distance(&new_mean) > TOLERANCE {
                changed = true;
            }

            means[i] = new_mean;
        }

        iters_left -= 1;

        if !changed || iters_left <= 0 {
            break;
        }
        iters_left -= 1;
    }

    // The length of every cluster divided by total pixels gives the dominance of each mean
    // For every mean, the corresponding dominance is added as a tuple item
    return clusters
        .iter()
        .enumerate()
        .map(|(i, cluster)| (means[i].clone(), cluster.len() as f32 / pixels.len() as f32))
        .collect();
}

/// Recalculates the means using a weight function
fn recal_means(colors: &Vec<LAB>, weight: WeightFn) -> LAB {
    let mut new_color = LAB {
        l: 0.0,
        a: 0.0,
        b: 0.0,
    };
    let mut w_sum = 0.0;

    for col in colors.iter() {
        let w = weight(col);
        w_sum += w;
        new_color.l += w * col.l;
        new_color.a += w * col.a;
        new_color.b += w * col.b;
    }

    new_color.l /= w_sum;
    new_color.a /= w_sum;
    new_color.b /= w_sum;

    return new_color;
}
