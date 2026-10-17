// Port of PixelUserAgentProvider.cs
use rand::seq::IndexedRandom;
pub const USER_AGENTS: &[&str] = &[
    "Dalvik/2.1.0 (Linux; U; Android 12; Pixel 3a Build/SP2A.220305.012)",
    "Dalvik/2.1.0 (Linux; U; Android 12; Pixel 3a Build/SP2A.220505.002)",
    "Dalvik/2.1.0 (Linux; U; Android 12; Pixel 4 Build/SQ3A.220705.003.A1)",
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 4 Build/TP1A.220624.014)",
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 4 Build/TP1A.221005.002)",
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 4a Build/TQ3A.230705.001)",
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 5 Build/TQ3A.230901.001)",
    "Dalvik/2.1.0 (Linux; U; Android 14; Pixel 5 Build/UP1A.231005.007)",
    "Dalvik/2.1.0 (Linux; U; Android 14; Pixel 5a Build/AP1A.240505.004)",
    "Dalvik/2.1.0 (Linux; U; Android 17; Pixel 6 Build/CP2A.260605.012)",
    "Dalvik/2.1.0 (Linux; U; Android 17; Pixel 6 Build/CP2A.260705.006)",
    "Dalvik/2.1.0 (Linux; U; Android 16; Pixel 7 Build/CP1A.260305.018)",
    "Dalvik/2.1.0 (Linux; U; Android 16; Pixel 7 Build/CP1A.260405.005.B1)",
    "Dalvik/2.1.0 (Linux; U; Android 17; Pixel 7 Build/CP2A.260605.012)",
    "Dalvik/2.1.0 (Linux; U; Android 13; Pixel 7 Build/TQ3A.230901.001)",
];
pub fn pick() -> &'static str { USER_AGENTS.choose(&mut rand::rng()).copied().unwrap_or(USER_AGENTS[0]) }
