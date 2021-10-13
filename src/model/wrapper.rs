use serde::ser::{Serialize, SerializeStruct, Serializer};
use twilight_model::{
    channel::{
        thread::{PrivateThread, PublicThread},
        GuildChannel, TextChannel,
    },
    guild::{Guild, Member, PartialGuild, PartialMember, Role},
    id::{ChannelId, GuildId, UserId},
    user::{CurrentUser, User},
};

pub struct GuildWrapper<'g>(pub &'g Guild);

impl<'g> From<&'g Guild> for GuildWrapper<'g> {
    fn from(guild: &'g Guild) -> Self {
        Self(guild)
    }
}

impl<'g> Serialize for GuildWrapper<'g> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 3 + self.0.icon.is_some() as usize;
        let mut guild = s.serialize_struct("CachedGuild", len)?;

        if let Some(ref icon) = self.0.icon {
            guild.serialize_field("a", icon)?;
        }

        guild.serialize_field("b", &self.0.id)?;
        guild.serialize_field("c", &self.0.name)?;
        guild.serialize_field("d", &self.0.owner_id)?;

        guild.end()
    }
}

pub struct PartialGuildWrapper<'g>(pub &'g PartialGuild);

impl<'g> From<&'g PartialGuild> for PartialGuildWrapper<'g> {
    fn from(guild: &'g PartialGuild) -> Self {
        Self(guild)
    }
}

impl<'g> Serialize for PartialGuildWrapper<'g> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 3 + self.0.icon.is_some() as usize;
        let mut guild = s.serialize_struct("CachedGuild", len)?;

        if let Some(ref icon) = self.0.icon {
            guild.serialize_field("a", icon)?;
        }

        guild.serialize_field("b", &self.0.id)?;
        guild.serialize_field("c", &self.0.name)?;
        guild.serialize_field("d", &self.0.owner_id)?;

        guild.end()
    }
}

pub struct CurrentUserWrapper<'u>(pub &'u CurrentUser);

impl<'u> From<&'u CurrentUser> for CurrentUserWrapper<'u> {
    fn from(user: &'u CurrentUser) -> Self {
        Self(user)
    }
}

impl<'u> Serialize for CurrentUserWrapper<'u> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 3 + self.0.avatar.is_some() as usize;
        let mut user = s.serialize_struct("CachedCurrentUser", len)?;

        if let Some(ref avatar) = self.0.avatar {
            user.serialize_field("a", avatar)?;
        }

        user.serialize_field("b", &self.0.discriminator)?;
        user.serialize_field("c", &self.0.id)?;
        user.serialize_field("d", &self.0.name)?;

        user.end()
    }
}

pub struct RoleWrapper<'r>(pub &'r Role);

impl<'r> From<&'r Role> for RoleWrapper<'r> {
    fn from(role: &'r Role) -> Self {
        Self(role)
    }
}

impl<'r> Serialize for RoleWrapper<'r> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut role = s.serialize_struct("CachedRole", 4)?;

        role.serialize_field("a", &self.0.id)?;
        role.serialize_field("b", &self.0.name)?;
        role.serialize_field("c", &self.0.permissions)?;
        role.serialize_field("d", &self.0.position)?;

        role.end()
    }
}

pub struct MemberWrapper<'m>(pub &'m Member);

impl<'m> From<&'m Member> for MemberWrapper<'m> {
    fn from(member: &'m Member) -> Self {
        Self(member)
    }
}

impl<'m> Serialize for MemberWrapper<'m> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 2 + self.0.nick.is_some() as usize + !self.0.roles.is_empty() as usize;
        let mut member = s.serialize_struct("CachedMember", len)?;

        member.serialize_field("a", &self.0.guild_id)?;

        if let Some(ref nick) = self.0.nick {
            member.serialize_field("b", nick)?;
        }

        if !self.0.roles.is_empty() {
            member.serialize_field("c", &self.0.roles)?;
        }

        member.serialize_field("d", &self.0.user.id)?;

        member.end()
    }
}

pub struct PartialMemberWrapper<'m> {
    guild: GuildId,
    member: &'m PartialMember,
    user: UserId,
}

impl<'m> From<(&'m PartialMember, GuildId, UserId)> for PartialMemberWrapper<'m> {
    fn from((member, guild, user): (&'m PartialMember, GuildId, UserId)) -> Self {
        Self {
            member,
            guild,
            user,
        }
    }
}

impl<'m> Serialize for PartialMemberWrapper<'m> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 2 + self.member.nick.is_some() as usize + !self.member.roles.is_empty() as usize;
        let mut member = s.serialize_struct("CachedMember", len)?;

        member.serialize_field("a", &self.guild)?;

        if let Some(ref nick) = self.member.nick {
            member.serialize_field("b", nick)?;
        }

        if !self.member.roles.is_empty() {
            member.serialize_field("c", &self.member.roles)?;
        }

        member.serialize_field("d", &self.user)?;

        member.end()
    }
}

pub struct UserWrapper<'u>(pub &'u User);

impl<'u> From<&'u User> for UserWrapper<'u> {
    fn from(user: &'u User) -> Self {
        Self(user)
    }
}

impl<'u> Serialize for UserWrapper<'u> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 3 + self.0.avatar.is_some() as usize + !self.0.bot as usize;
        let mut user = s.serialize_struct("CachedUser", len)?;

        if let Some(ref avatar) = self.0.avatar {
            user.serialize_field("a", avatar)?;
        }

        if !self.0.bot {
            user.serialize_field("b", &self.0.bot)?;
        }

        user.serialize_field("c", &self.0.discriminator)?;
        user.serialize_field("d", &self.0.id)?;
        user.serialize_field("e", &self.0.name)?;

        user.end()
    }
}

pub struct TextChannelWrapper<'c>(pub &'c TextChannel);

impl<'c> From<&'c TextChannel> for TextChannelWrapper<'c> {
    fn from(channel: &'c TextChannel) -> Self {
        Self(channel)
    }
}

impl<'c> Serialize for TextChannelWrapper<'c> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 2
            + self.0.guild_id.is_some() as usize
            + !self.0.permission_overwrites.is_empty() as usize;

        let mut channel = s.serialize_struct("CachedTextChannel", len)?;

        if let Some(ref guild) = self.0.guild_id {
            channel.serialize_field("a", guild)?;
        }

        channel.serialize_field("b", &self.0.id)?;
        channel.serialize_field("c", &self.0.name)?;

        if !self.0.permission_overwrites.is_empty() {
            channel.serialize_field("d", &self.0.permission_overwrites)?;
        }

        channel.end()
    }
}

pub struct PublicThreadWrapper<'c>(pub &'c PublicThread);

impl<'c> From<&'c PublicThread> for PublicThreadWrapper<'c> {
    fn from(channel: &'c PublicThread) -> Self {
        Self(channel)
    }
}

impl<'c> Serialize for PublicThreadWrapper<'c> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 2 + self.0.guild_id.is_some() as usize + self.0.parent_id.is_some() as usize;
        let mut channel = s.serialize_struct("CachedThread", len)?;

        if let Some(ref guild) = self.0.guild_id {
            channel.serialize_field("a", guild)?;
        }

        channel.serialize_field("b", &self.0.id)?;
        channel.serialize_field("c", &self.0.name)?;

        if let Some(ref parent_id) = self.0.parent_id {
            channel.serialize_field("d", parent_id)?;
        }

        channel.end()
    }
}

pub struct PrivateThreadWrapper<'c>(pub &'c PrivateThread);

impl<'c> From<&'c PrivateThread> for PrivateThreadWrapper<'c> {
    fn from(channel: &'c PrivateThread) -> Self {
        Self(channel)
    }
}

impl<'c> Serialize for PrivateThreadWrapper<'c> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let len = 2 + self.0.guild_id.is_some() as usize + self.0.parent_id.is_some() as usize;
        let mut channel = s.serialize_struct("CachedThread", len)?;

        if let Some(ref guild) = self.0.guild_id {
            channel.serialize_field("a", guild)?;
        }

        channel.serialize_field("b", &self.0.id)?;
        channel.serialize_field("c", &self.0.name)?;

        if let Some(ref parent_id) = self.0.parent_id {
            channel.serialize_field("d", parent_id)?;
        }

        channel.end()
    }
}

pub enum BasicGuildChannel<'c> {
    PrivateThread(&'c PrivateThread),
    PublicThread(&'c PublicThread),
    Text(&'c TextChannel),
}

impl<'c> BasicGuildChannel<'c> {
    pub const fn guild_id(&self) -> Option<GuildId> {
        match self {
            Self::PrivateThread(c) => c.guild_id,
            Self::PublicThread(c) => c.guild_id,
            Self::Text(c) => c.guild_id,
        }
    }

    pub const fn id(&self) -> ChannelId {
        match self {
            Self::PrivateThread(c) => c.id,
            Self::PublicThread(c) => c.id,
            Self::Text(c) => c.id,
        }
    }

    pub fn from(channel: &'c GuildChannel) -> Option<Self> {
        match channel {
            GuildChannel::PrivateThread(c) => Some(Self::PrivateThread(c)),
            GuildChannel::PublicThread(c) => Some(Self::PublicThread(c)),
            GuildChannel::Text(c) => Some(Self::Text(c)),
            _ => None,
        }
    }
}

impl<'c> Serialize for BasicGuildChannel<'c> {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        match self {
            BasicGuildChannel::PrivateThread(c) => {
                s.serialize_newtype_variant("CachedGuildChannel", 0, "a", &PrivateThreadWrapper(c))
            }
            BasicGuildChannel::PublicThread(c) => {
                s.serialize_newtype_variant("CachedGuildChannel", 1, "b", &PublicThreadWrapper(c))
            }
            BasicGuildChannel::Text(c) => {
                s.serialize_newtype_variant("CachedGuildChannel", 2, "c", &TextChannelWrapper(c))
            }
        }
    }
}
