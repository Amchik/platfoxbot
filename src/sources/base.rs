/// Generic channel post, that can be fetched from `PostFabric`s
/// and putted to dest
#[derive(Clone)]
pub struct ChannelPost {
    /// Post text. May be empty string if not exists.
    pub text: String,
    /// Post media (photo, video, etc).
    /// May be empty
    pub media: Vec<ChannelPostMedia>,
    /// Post source. Please use `<SOCIAL> // <GROUP>` format.
    /// Example: `twitter // OnlyFlans`
    pub source: String,
}

/// Generic media in channel post.
#[derive(Clone)]
pub enum ChannelPostMedia {
    /// Photo. .png, .jpeg or etc... (NOT GIF)
    Photo(String),
    /// Video or GIF. .mp4 and .gif only, please
    Video(String),
}

pub trait PostFabric {
    /// ID of last post.
    /// If no must be 0u64
    fn last_id(&self) -> u64;

    /// Fetch last posts after `last_id`.
    /// Returns empty vec on fail
    fn fetch_last_posts(&self) -> Vec<ChannelPost>;
}

