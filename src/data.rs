pub struct Project {
    pub name: &'static str,
    pub date: &'static str,
    pub description: &'static str,
    pub tech: &'static str,
    pub link: &'static str,
    pub image: &'static str,
    #[allow(dead_code)]
    pub repo: Option<&'static str>,
}

pub const PROJECTS: &[Project] = &[
    Project {
        name: "ByteMate",
        date: "August 2025",
        description: "A Discord bot written in Rust.",
        tech: "Rust, dioxus, serenity, poise, byteorm",
        link: "https://bytemate.xyz",
        image: "https://bytemate.xyz/og-image.png",
        repo: None,
    },
    Project {
        name: "ByteORM",
        date: "November 2025",
        description: "An async ORM for Rust.",
        tech: "Rust, pest",
        link: "https://github.com/bytematebot/byteORM",
        image: "/assets/repos/byteorm.png",
        repo: Some("bytematebot/byteORM"),
    },
    Project {
        name: "Dioxus Press",
        date: "August 2026",
        description: "A docs framework for Dioxus.",
        tech: "Rust, dioxus",
        link: "https://github.com/bytematebot/dioxus-press",
        image: "/assets/repos/dioxus-press.png",
        repo: Some("bytematebot/dioxus-press"),
    },
    Project {
        name: "raskell",
        date: "August 2026",
        description: "Haskell-style do notation for Rust, as macros.",
        tech: "Rust, proc macros",
        link: "https://github.com/bytematebot/raskell",
        image: "/assets/repos/raskell.png",
        repo: Some("bytematebot/raskell"),
    },
    Project {
        name: "Yawn",
        date: "November 2026",
        description: "A full-stack web framework for Rust.",
        tech: "Rust, wasm",
        link: "https://github.com/yawn-rs/yawn",
        image: "/assets/repos/yawn.png",
        repo: Some("yawn-rs/yawn"),
    },
];

pub struct Post {
    pub slug: &'static str,
    pub title: &'static str,
    pub date: &'static str,
    pub draft: bool,
    pub body: &'static str,
}

pub const POSTS: &[Post] = &[Post {
    slug: "how-hdo-works",
    title: "What hdo! expands to",
    date: "2026-10-14",
    draft: false,
    body: include_str!("../posts/how-hdo-works.md"),
}];

pub struct PersonalInfo {
    pub name: &'static str,
    #[allow(dead_code)]
    pub bio: &'static str,
    pub email: &'static str,
    pub github: &'static str,
    pub discord: &'static str,
}

pub const PERSONAL_INFO: PersonalInfo = PersonalInfo {
    name: "meetzli",
    bio: "Software developer building libraries and tools in Rust.",
    email: "hello@meetz.li",
    github: "https://github.com/meetzli",
    discord: "https://discord.com/users/1145840021954375781",
};
