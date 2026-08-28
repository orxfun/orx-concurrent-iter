use orx_criterion::Factors;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ComputeType {
    /// Very light arithmetic / bitwise operations (~few ns).
    /// Highly sensitive to lock contention and pull frequency.
    Light,
    /// Moderate CPU work (~0.5 - 1 µs per item).
    Medium,
    /// Heavy CPU work (~5 - 10 µs per item).
    Heavy,
    /// Variable/skewed workload (mix of light and heavy items).
    /// Tests dynamic work distribution and thread load balancing.
    Variable,
    /// Allocates heap structures (`String` and nested `Vec`s).
    /// Tests heap allocation pressure and moving structured data across threads.
    Alloc,
}

impl ComputeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Medium => "medium",
            Self::Heavy => "heavy",
            Self::Variable => "variable",
            Self::Alloc => "alloc",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelineType {
    /// `filter(predicate).map(transformation)`: filters ~50% of elements.
    /// The iterator `.next()` evaluated inside the lock must skip non-matching items.
    FilterMap,
    /// `map(transformation)`: transforms all elements without filtering.
    Map,
}

impl PipelineType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FilterMap => "filter_map",
            Self::Map => "map",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct InputVariant {
    pub n: usize,
    pub compute: ComputeType,
    pub pipeline: PipelineType,
}

impl Factors for InputVariant {
    fn factor_names() -> Vec<&'static str> {
        vec!["n", "compute", "pipeline"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            format_size(self.n),
            self.compute.as_str().to_string(),
            self.pipeline.as_str().to_string(),
        ]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        vec![
            format_size(self.n),
            self.compute.as_str().to_string(),
            match self.pipeline {
                PipelineType::FilterMap => "fmap".to_string(),
                PipelineType::Map => "map".to_string(),
            },
        ]
    }
}

fn format_size(n: usize) -> String {
    if n >= 1_000_000 && n % 1_000_000 == 0 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 && n % 1_000 == 0 {
        format!("{}k", n / 1_000)
    } else if n >= 1024 && n.is_power_of_two() {
        let exp = n.trailing_zeros();
        format!("2e{}", exp)
    } else {
        n.to_string()
    }
}
