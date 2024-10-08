//! Finds an optimal local scheme using ILP.
//!
//! 1. For each lmer, create w boolean variables, 1 of which must be true.
//! 2. Create a De Bruijn word of sufficient order, and a boolean variable per kmer.
//! 3. Each kmer selected by the local scheme must be true.
//! 4. Minimizer the number of selected kmers.

use itertools::repeat_n;

use good_lp::*;
use itertools::Itertools;

use crate::{de_bruijn_seq::cyclic_exact_density_string, pack, ExplicitLocalScheme};

pub fn best_local_scheme(
    k: usize,
    w: usize,
    sigma: usize,
    forward: bool,
) -> ((usize, usize), ExplicitLocalScheme) {
    let l = k + w - 1;
    let lmers = repeat_n(0..sigma, l)
        .multi_cartesian_product()
        .collect_vec();

    let mut problem = ProblemVariables::new();

    // var[i][j] = 1 if the i'th lmer selects position j.
    let lmer_vars: Vec<Vec<_>> = (0..lmers.len())
        .map(|_| {
            (0..w)
                .map(|_| problem.add(variable().integer().bounds(0..1)))
                .collect()
        })
        .collect();

    // len sigma^order + (k-1)
    let (text, chars) = cyclic_exact_density_string(k, w, sigma, forward);
    // eprintln!("#text: {text:?}");
    eprintln!("dBG len: {chars}");
    // double the text.

    let selected_vars = (0..chars)
        .map(|_| problem.add(variable().integer().bounds(0..1)))
        .collect_vec();
    eprintln!("#selected_vars: {}", selected_vars.len());

    let mut objective = Expression::default();
    for selected in &selected_vars {
        objective += selected;
    }

    let mut problem = problem.minimise(objective.clone()).using(default_solver);

    let num_vars = lmer_vars.len() + selected_vars.len();
    eprintln!("#vars: {num_vars}");
    let mut num_constraints = 0;

    // Select 1 per lmer.
    for lmer_var in &lmer_vars {
        problem.add_constraint(lmer_var.iter().sum::<Expression>().eq(1));
        num_constraints += 1;
    }

    // Set selected kmers.
    for (i, lmer) in text.windows(l).take(chars).enumerate() {
        let lmer_id = pack(lmer, sigma);
        for (j, lmer_var) in lmer_vars[lmer_id].iter().enumerate() {
            let selected_var = &selected_vars[(i + j) % chars];
            // eprintln!("{:?} <= {:?}", lmer_var, selected_var);
            problem.add_constraint(lmer_var.into_expression().leq(selected_var));
            num_constraints += 1;
        }
    }

    if forward {
        // let x0..x(w-1) and y0..y(w-1) be the variables for consecutive lmers.
        //
        // x0 x1 x2
        //    y0 y1 y2
        // y0 <= x0 + x1, i.e. if x0=0 and x1=0, then y0=0.
        //
        // Thus:
        // yi <= x0 + x1 + ... + x(i+1) for all 0<=i<w-2. (i=w-2 is redundant.)
        for (lmer1, lmer2) in text.windows(l).tuple_windows().take(chars) {
            let lmer1_id = pack(lmer1, sigma);
            let lmer2_id = pack(lmer2, sigma);
            for i in 0..w - 2 {
                let sum_xi = lmer_vars[lmer1_id][..=i + 1]
                    .iter()
                    .map(|var| var.into_expression())
                    .sum::<Expression>();
                let yi = &lmer_vars[lmer2_id][i];
                problem.add_constraint(yi.into_expression().leq(sum_xi));
                num_constraints += 1;
            }
        }
    }

    eprintln!("#constraints: {num_constraints}");

    let solution = problem.solve().unwrap();
    let selected = solution.eval(&objective);
    eprintln!("#selected: {selected}");

    let ls = ExplicitLocalScheme {
        k,
        w,
        sigma,
        map: lmer_vars
            .iter()
            .map(|lmer_vars| {
                lmer_vars
                    .iter()
                    .enumerate()
                    .find(|(_i, var)| solution.value(**var).round() == 1.0)
                    .unwrap()
                    .0 as u8
            })
            .collect(),
    };

    ((selected as usize, chars), ls)
}
