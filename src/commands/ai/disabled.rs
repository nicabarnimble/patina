//! Error-only compatibility names; no guest executable detection or launching.
#[derive(Debug, Clone, clap::Subcommand)]
pub enum Commands {
    /// Requires --features full; guest launching belongs to PiMux.
    Setup {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full; guest launching belongs to PiMux.
    Refresh {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full; guest launching belongs to PiMux.
    Claude {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full; guest launching belongs to PiMux.
    #[command(name = "opencode")]
    OpenCode {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full; guest launching belongs to PiMux.
    Gemini {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
    /// Requires --features full; guest launching belongs to PiMux.
    Pi {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<std::ffi::OsString>,
    },
}
impl Commands {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Setup { .. } => "setup",
            Self::Refresh { .. } => "refresh",
            Self::Claude { .. } => "claude",
            Self::OpenCode { .. } => "opencode",
            Self::Gemini { .. } => "gemini",
            Self::Pi { .. } => "pi",
        }
    }
}
