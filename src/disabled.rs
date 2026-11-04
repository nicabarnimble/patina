//! Error-only CLI compatibility surface. Never imports legacy implementations.
#[derive(Debug, clap::Subcommand)]
pub enum Commands {
    /// Requires --features full (not included in the knowledge build).
    Doctor {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    #[command(visible_alias = "plugin")]
    Child {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Connect {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Lake {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Mother {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Pando {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Secrets {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Serve {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Interface {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Slate {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full (not included in the knowledge build).
    Spec {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
}
impl Commands {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Doctor { .. } => "doctor",
            Self::Child { .. } => "child",
            Self::Connect { .. } => "connect",
            Self::Lake { .. } => "lake",
            Self::Mother { .. } => "mother",
            Self::Pando { .. } => "pando",
            Self::Secrets { .. } => "secrets",
            Self::Serve { .. } => "serve",
            Self::Interface { .. } => "interface",
            Self::Slate { .. } => "slate",
            Self::Spec { .. } => "spec",
        }
    }
}
