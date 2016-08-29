use serde_json::Value;
use std::i64;

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
enum Availability {
    /// Only those with the 'Bot Commander' role can use the command
    BotCommander,
    /// No one can use the command
    Disabled,
    /// Everyone can use the command
    Enabled,
}

impl Availability {
    pub fn from_num(num: u8) -> Option<Availability> {
        match num {
            0 => Some(Availability::Disabled),
            1 => Some(Availability::Enabled),
            2 => Some(Availability::BotCommander),
            _ => None,
        }
    }

    pub fn from_str(name: &str) -> Option<Availability> {
        match name {
            "commander" | "2" => Some(Availability::BotCommander),
            "disabled" | "0" => Some(Availability::Disabled),
            "enabled" | "1" => Some(Availability::Enabled),
            _ => None,
        }
    }

    pub fn num(&self) -> i64 {
        match *self {
            Availability::Disabled => 0,
            Availability::Enabled => 1,
            Availability::BotCommander => 2,
        }
    }
}

#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Ord, PartialOrd)]
enum ConfigType {
    Availability,
    Int,
    String,
}

impl ConfigType {
    pub fn from_num(num: u8) -> Option<ConfigType> {
        match num {
            0 => Some(ConfigType::Availability),
            1 => Some(ConfigType::Int),
            2 => Some(ConfigType::String),
            _ => None,
        }
    }
}

macro_rules! config {
    ($name:ident, $key:expr, $kind:path, $default:expr, $desc:expr) => {
        /// $desc
        #[derive(Clone, Debug)]
        struct $name {
            default: Value,
            description: String,
            key: String,
            max_value: Option<i64>,
            min_value: Option<i64>,
            kind: ConfigType,
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    default: $default,
                    description: String::from($desc),
                    key: String::from($key),
                    max_value: None,
                    min_value: None,
                    kind: $kind,
                }
            }
        }
    };

    ($name:ident, $key:expr, $kind:path, $default:expr, $min:expr, $max:expr, $desc:expr) => {
        /// $desc
        #[derive(Clone, Debug)]
        struct $name {
            default: Value,
            description: String,
            key: String,
            max_value: Option<i64>,
            min_value: Option<i64>,
            kind: ConfigType,
        }

        impl $name {
            pub fn new() -> $name {
                $name {
                    default: $default,
                    description: String::from($desc),
                    key: String::from($key),
                    max_value: Some($max),
                    min_value: Some($min),
                    kind: $kind,
                }
            }
        }
    };
}

config! {
    AestheticAvailable,
    "aesthetic.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `aesthetic` is available."
}

config! {
    AestheticCapsAvailable,
    "aesthetic.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `aesthetic` is available."
}

config! {
    AesAvailable,
    "aes.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `aes` is available."
}

config! {
    AesCapsAvailable,
    "aescaps.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `aescaps` is available."
}

config! {
    AnimeAvailable,
    "anime.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `anime` is available."
}

config! {
    ChannelInfoAvailable,
    "channelinfo.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `channelinfo` is available."
}

config! {
    CoinflipAvailableAvailable,
    "coinflip.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `coinflip` is available."
}

config! {
    CoinflipSide,
    "coinflip.side",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the coin can land on its side."
}

config! {
    ConversationAvailable,
    "conversation.available",
    ConfigType::Availability,
    Value::I64(Availability::Disabled.num()),
    "Whether the ability to converse with nano is available.

    The command `q` is a command to converse with an AI. This allows users to
    ask questions, discover what an electron is, determine whether steel
    memes melt stale memes, and just about basically everything else."
}

config! {
    DefineAvailable,
    "define.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `define` is available."
}

config! {
    DefineExample,
    "define.example",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether to display an example with the definition."
}

config! {
    HelloAvailable,
    "hello.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `hello` is available."
}

config! {
    LmgtfyAvailable,
    "lmgtfy.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `lmgtfy` is available."
}

config! {
    LmgtfyResults,
    "lmgtfy.results",
    ConfigType::Int,
    Value::I64(1),
    1,
    5,
    "The number of results to return from a search."
}

config! {
    MagicEightBallAvailable,
    "8ball.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `8ball` is available."
}

config! {
    MangaAvailable,
    "manga.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `manga` is available."
}

config! {
    MfwAvailable,
    "mfw.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `mfw` is available."
}

config! {
    PiAvailable,
    "pi.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `pi` is available."
}

config! {
    PiPrecisionDefault,
    "pi.precision.default",
    ConfigType::Int,
    Value::I64(100),
    0,
    1000,
    "The number of digits of pi to return by default."
}

config! {
    PiPrecisionMaximum,
    "pi.precision.maximum",
    ConfigType::Int,
    Value::I64(100),
    1000,
    0,
    "The number of digits of pi that can be returned at a maximum amount.

    This is useful to prevent a serve rmessage flood (1000 is a lot of text)."
}

config! {
    PingAvailable,
    "ping.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `ping` is available."
}

config! {
    PixivAutomatic,
    "pixiv.automatic",
    ConfigType::Availability,
    Value::I64(Availability::Disabled.num()),
    "Whether to automatically embed an image when a pixiv link is seen.

    This will automatically retrieve the pixiv image whenever a pixiv link is
    seen _anywhere_ in _any_ message, regardless if the `pixiv` command is
    used."
}

config! {
    PixivAvailable,
    "pixiv.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `pixiv` is available."
}

config! {
    PixivInfo,
    "pixiv.info",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether to embed author information at the bottom of the picture.

    Enabling this will give a white bar at the bottom of the image with a URL to
    the illustration and the author."
}

config! {
    RandomAvailable,
    "random.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `random` is available."
}

config! {
    RemindMeAvailable,
    "remindme.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `remindme` and `remind` are available."
}

config! {
    RoleInfoAvailable,
    "roleinfo.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `roleinfo` is available."
}

config! {
    RollAvailable,
    "rol..available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `roll` is available."
}

config! {
    RollCustom,
    "roll.custom",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether users can give custom numbers to role between.

    Otherwise, a predetermined set of numbers will be rolled."
}

config! {
    RollMax,
    "roll.max",
    ConfigType::Int,
    Value::I64(i64::MAX),
    i64::MIN + 1,
    i64::MAX,
    "The maximum value that can be rolled."
}

config! {
    RollMinimum,
    "roll.minimum",
    ConfigType::Int,
    Value::I64(i64::MIN),
    i64::MIN,
    i64::MAX - 1,
    "The minimum value that can be rolled."
}

config! {
    ServerInfoAvailable,
    "serverinfo.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `serverinfo` is available."
}

config! {
    SkipAvailable,
    "skip.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `skip` is available."
}

config! {
    SkipRequired,
    "skip.required",
    ConfigType::Int,
    Value::I64(2),
    1,
    50,
    "The number of skip votes required for a song to be skipped."
}

config! {
    StatsAvailable,
    "stats.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `stats` is available."
}

config! {
    TagsAvailable,
    "tags.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use the tagging system is available."
}

config! {
    TeamsAvailable,
    "teams.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `teams` is available."
}

config! {
    UserInfoAvailable,
    "userinfo.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `userinfo` is available."
}

config! {
    WeatherAvailable,
    "weather.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `weather` is available."
}

config! {
    WeatherSaving,
    "weather.saving",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether users can save their location to this server.

    This will allow users to easily retrieve their weather via just `weather`."
}

config! {
    WolframAvailable,
    "wolfram.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `wolfram` is available."
}

config! {
    WolframRows,
    "wolfram.rows",
    ConfigType::Int,
    Value::I64(2),
    1,
    10,
    "The number of rows of data to output from Wolfram."
}

config! {
    XkcdAvailable,
    "xkcd.available",
    ConfigType::Availability,
    Value::I64(Availability::Enabled.num()),
    "Whether the ability to use `xkcd` is available."
}
