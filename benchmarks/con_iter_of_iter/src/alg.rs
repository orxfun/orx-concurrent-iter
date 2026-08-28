use orx_criterion::Factors;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Seq,
    Rayon,
    ConIterSingle,
    ConIterChunk(usize),
}

impl Method {
    #[allow(unreachable_code)]
    pub fn get() -> Self {
        #[cfg(feature = "seq")]
        return Self::Seq;

        #[cfg(feature = "rayon")]
        return Self::Rayon;

        #[cfg(feature = "con-iter-single")]
        return Self::ConIterSingle;

        #[cfg(feature = "con-iter-c16")]
        return Self::ConIterChunk(16);

        #[cfg(feature = "con-iter-c64")]
        return Self::ConIterChunk(64);

        #[cfg(feature = "con-iter-c256")]
        return Self::ConIterChunk(256);

        #[cfg(feature = "con-iter-c1024")]
        return Self::ConIterChunk(1024);

        panic!("must add one of the algorithm variants as feature");
    }
}

impl Factors for Method {
    fn factor_names() -> Vec<&'static str> {
        vec!["method"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon => "rayon".to_string(),
            Self::ConIterSingle => "con-iter-single".to_string(),
            Self::ConIterChunk(c) => format!("con-iter-c{c}"),
        }]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        vec![match self {
            Self::Seq => "seq".to_string(),
            Self::Rayon => "rayon".to_string(),
            Self::ConIterSingle => "con-single".to_string(),
            Self::ConIterChunk(c) => format!("con-c{c}"),
        }]
    }
}
