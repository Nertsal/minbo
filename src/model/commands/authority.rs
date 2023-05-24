#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum AuthorityLevel {
    Viewer,
    Subscriber,
    Moderator,
    Broadcaster,
    /// The host machine running the bot.
    Host,
}

impl AuthorityLevel {
    pub fn from_badges(badges: &[twitch_irc::message::Badge]) -> Self {
        badges
            .iter()
            .fold(AuthorityLevel::Viewer, |authority_level, badge| {
                authority_level.max(AuthorityLevel::from_badge(badge))
            })
    }

    pub fn from_badge(badge: &twitch_irc::message::Badge) -> Self {
        match badge.name.as_str() {
            "subscriber" => AuthorityLevel::Subscriber,
            "broadcaster" => AuthorityLevel::Broadcaster,
            "moderator" => AuthorityLevel::Moderator,
            _ => AuthorityLevel::Viewer,
        }
    }
}
