use crate::genome::Genome;

use rand::prelude::*;

const ELITES: i64 = 10;
const POOL_SIZE: i64 = 80;
const TOURNAMENT_SIZE: usize = 30; // 1 is random, higher up to pop.len() is higher pressure
const SELECTION_PRESSURE: f64 = 0.9; // Higher = closer to deterministic, should be between 0 and 1

pub fn _fitness_selection(old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut rng = thread_rng();
    let mut pool: Vec<Genome> = Vec::new();
    let mut fitness_cdf: Vec<f64> = Vec::new();

    if ELITES > 0 {
        let l = old_pop.len();
        for i in 0..ELITES {
            new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
        }
    }

    // make list of increasing fitness
    let mut acc = 0.0;
    for g in old_pop.iter() {
        acc += g.get_fitness();
        fitness_cdf.push(acc);
    }
    let fit_total = acc;

    while pool.len() < POOL_SIZE as usize {
        let mut rand: f64 = rng.gen();
        rand = rand * fit_total;
        pool.push(old_pop[_find(rand, &fitness_cdf)].clone());
    }
    
    return pool;
}

pub fn tournament_selection(mut old_pop: Vec<Genome>, new_gen: &mut Vec<Genome>) -> Vec<Genome> {
    let mut pool = Vec::new();
    let mut rng = thread_rng();

    if ELITES > 0 {
        let l = old_pop.len();
        for i in 0..ELITES {
            new_gen.push((*old_pop.get(l - 1 - i as usize).unwrap()).clone());
        }
    }

    for _ in 0..POOL_SIZE {
        
        let mut participants: Vec<usize> = Vec::new();
        for _ in 0..TOURNAMENT_SIZE {
            participants.push(rng.gen_range(0, old_pop.len()));
        }
        participants.sort(); // Assumption : old_pop is sorted
        
        let winner: f64 = rng.gen();
        let mut acc = 0.0;
        let mut k = 0;
        while k < TOURNAMENT_SIZE && acc < winner {
            acc = acc + SELECTION_PRESSURE*(1.0 - SELECTION_PRESSURE).powi(k as i32);
            k = k + 1;
        }
        k = TOURNAMENT_SIZE - k;

        pool.push(old_pop.remove(participants[k]));
    }
    return pool
}

fn _find(elem: f64, list: &Vec<f64>) -> usize {
    let mut low = 0;
    let mut high = list.len() - 1;
    loop {
        let middle = (low+high)/2;
        if middle == low {
            return high
        }
        if elem < list[middle] {
            high = middle;
        }
        else {
            low = middle;
        }
    }
}