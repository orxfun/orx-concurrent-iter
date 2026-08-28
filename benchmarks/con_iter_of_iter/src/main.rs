mod alg;
mod exp;
mod input;

use crate::alg::Method;
use crate::exp::Exp;
use crate::input::{ComputeType, InputVariant, PipelineType};
use bench_helper::{runner, BenchArgs};
use clap::Parser;

fn main() {
    let args = BenchArgs::parse();

    let ns = [16_384, 65_536];
    let computes = [
        ComputeType::Light,
        ComputeType::Medium,
        ComputeType::Heavy,
        ComputeType::Variable,
        ComputeType::Alloc,
    ];
    let pipelines = [PipelineType::FilterMap, PipelineType::Map];

    let mut input_variants = Vec::new();
    for n in ns {
        for compute in computes {
            for pipeline in pipelines {
                input_variants.push(InputVariant {
                    n,
                    compute,
                    pipeline,
                });
            }
        }
    }

    runner::run(&args, Exp, &input_variants, &Method::get());
}
