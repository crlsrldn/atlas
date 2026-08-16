//! Which Infuse connection mode is talking to us.
//!
//! Infuse announces its mode in the User-Agent — `Infuse-Direct`,
//! `Infuse-Library`, or `Infuse-Download` — which matters because the modes make
//! very different demands. Direct Mode fetches on demand and suits a virtual
//! catalogue; Library Mode tries to enumerate the whole library up front, which
//! against Atlas means paging a catalogue that has no natural end.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientMode {
    /// Infuse Direct Mode: on-demand fetches. The mode Atlas is built for.
    InfuseDirect,
    /// Infuse Library Mode: full pre-sync. Served, but bounded.
    InfuseLibrary,
    /// Infuse offline download.
    InfuseDownload,
    /// Some other Jellyfin client. Treated like Direct.
    Other,
}

impl ClientMode {
    pub fn from_user_agent(user_agent: Option<&str>) -> Self {
        let Some(agent) = user_agent else {
            return ClientMode::Other;
        };
        let agent = agent.to_ascii_lowercase();

        if agent.contains("infuse-library") {
            ClientMode::InfuseLibrary
        } else if agent.contains("infuse-download") {
            ClientMode::InfuseDownload
        } else if agent.contains("infuse") {
            ClientMode::InfuseDirect
        } else {
            ClientMode::Other
        }
    }

    /// Whether this mode will try to walk the entire catalogue.
    pub fn enumerates_library(self) -> bool {
        matches!(self, ClientMode::InfuseLibrary)
    }

    /// A short label for telemetry, so the Stremio and Infuse surfaces can be
    /// compared separately.
    pub fn label(self) -> &'static str {
        match self {
            ClientMode::InfuseDirect => "infuse_direct",
            ClientMode::InfuseLibrary => "infuse_library",
            ClientMode::InfuseDownload => "infuse_download",
            ClientMode::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClientMode;

    #[test]
    fn recognises_each_infuse_mode() {
        assert_eq!(
            ClientMode::from_user_agent(Some("Infuse-Direct/7.7")),
            ClientMode::InfuseDirect
        );
        assert_eq!(
            ClientMode::from_user_agent(Some("Infuse-Library/7.7")),
            ClientMode::InfuseLibrary
        );
        assert_eq!(
            ClientMode::from_user_agent(Some("Infuse-Download/7.7")),
            ClientMode::InfuseDownload
        );
    }

    #[test]
    fn library_mode_is_the_only_one_that_enumerates() {
        assert!(ClientMode::from_user_agent(Some("Infuse-Library/8.4")).enumerates_library());
        assert!(!ClientMode::from_user_agent(Some("Infuse-Direct/8.4")).enumerates_library());
        assert!(!ClientMode::from_user_agent(Some("Swiftfin")).enumerates_library());
    }

    #[test]
    fn unknown_and_missing_agents_fall_back_to_other() {
        assert_eq!(ClientMode::from_user_agent(None), ClientMode::Other);
        assert_eq!(
            ClientMode::from_user_agent(Some("Swiftfin/1.0")),
            ClientMode::Other
        );
    }

    #[test]
    fn matching_is_case_insensitive() {
        assert_eq!(
            ClientMode::from_user_agent(Some("INFUSE-LIBRARY/8.4")),
            ClientMode::InfuseLibrary
        );
    }
}
