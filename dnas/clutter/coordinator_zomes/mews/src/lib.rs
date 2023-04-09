use hdk::prelude::*;
use mews_integrity::*;
use regex::Regex;
use trust_atom_types::{QueryMineInput, TrustAtomInput};

fn get_my_mews_base(base_type: &str, ensure: bool) -> ExternResult<EntryHash> {
    let me: AgentPubKey = agent_info()?.agent_latest_pubkey;
    get_mews_base(me.into(), base_type, ensure)
}

fn get_mews_base(agent: AgentPubKey, base_type: &str, _ensure: bool) -> ExternResult<EntryHash> {
    let path = Path::from(format!("agent.{}.{}", agent, base_type));
    let anchor_hash = path.path_entry_hash()?;
    Ok(anchor_hash)
}

// *** Creating mews ***
#[hdk_extern]
pub fn create_mew(mew: CreateMewInput) -> ExternResult<ActionHash> {
    let mew_action_hash = match mew.mew_type {
        MewType::Original => match mew.text {
            Some(mew_string) => create_original_mew(MewContent {
                text: mew_string,
                links: mew.links,
            })?,
            None => {
                return Err(wasm_error!(WasmErrorInner::Guest(
                    "mew must contain text".to_string()
                )))
            }
        },
        MewType::Reply(original_action_hash) => match mew.text {
            Some(mew_string) => create_reply(
                MewContent {
                    text: mew_string,
                    links: mew.links,
                },
                original_action_hash,
            )?,
            None => {
                return Err(wasm_error!(WasmErrorInner::Guest(
                    "reply mew must contain text".to_string()
                )))
            }
        },
        MewType::MewMew(original_action_hash) => match mew.text {
            Some(_) => {
                return Err(wasm_error!(WasmErrorInner::Guest(
                    "mewmew must not contain text".to_string()
                )))
            }
            None => create_mewmew(original_action_hash)?,
        },
        MewType::Quote(original_entry_hash) => match mew.text {
            Some(mew_string) => create_quote(
                MewContent {
                    text: mew_string,
                    links: mew.links,
                },
                original_entry_hash,
            )?,
            None => {
                return Err(wasm_error!(WasmErrorInner::Guest(
                    "quote must contain text".to_string()
                )))
            }
        },
    };
    Ok(mew_action_hash)
}

pub fn create_original_mew(mew_content: MewContent) -> ExternResult<ActionHash> {
    let mew = Mew {
        mew_type: MewType::Original,
        content: Some(mew_content.clone()),
    };
    let mew_action_hash = create_entry(EntryTypes::Mew(mew))?;

    let base = get_my_mews_base(MEW_PATH_SEGMENT, true)?;

    let _link_ah = create_link(base, mew_action_hash.clone(), LinkTypes::Mew, ())?;
    parse_mew_text(mew_content, mew_action_hash.clone())?;
    Ok(mew_action_hash)
}

pub fn create_reply(
    mew_content: MewContent,
    original_action_hash: ActionHash,
) -> ExternResult<ActionHash> {
    let mew = Mew {
        mew_type: MewType::Reply(original_action_hash.clone()),
        content: Some(mew_content.clone()),
    };
    let reply_action_hash = create_entry(EntryTypes::Mew(mew))?;

    let base = get_my_mews_base(MEW_PATH_SEGMENT, true)?;

    let _link_ah = create_link(base, reply_action_hash.clone(), LinkTypes::Mew, ())?;
    // link off original entry as reply
    let _reply_link_ah = create_link(
        original_action_hash.clone(),
        reply_action_hash.clone(),
        LinkTypes::Reply,
        LinkTag::new(REPLY_PATH_SEGMENT),
    )?;
    parse_mew_text(mew_content, reply_action_hash.clone())?;
    Ok(reply_action_hash)
}

pub fn create_mewmew(original_action_hash: ActionHash) -> ExternResult<ActionHash> {
    let mew = Mew {
        mew_type: MewType::MewMew(original_action_hash.clone().into()),
        content: None,
    };
    let mewmew_action_hash = create_entry(EntryTypes::Mew(mew))?;

    let base = get_my_mews_base(MEW_PATH_SEGMENT, true)?;

    let _link_ah = create_link(base, mewmew_action_hash.clone(), LinkTypes::Mew, ())?;
    // link off original entry as mewmew
    let _quote_link_ah = create_link(
        original_action_hash.clone(),
        mewmew_action_hash.clone(),
        LinkTypes::Mewmew,
        LinkTag::new(MEWMEW_PATH_SEGMENT),
    )?;
    Ok(mewmew_action_hash)
}

pub fn create_quote(
    mew_content: MewContent,
    original_action_hash: ActionHash,
) -> ExternResult<ActionHash> {
    let mew = Mew {
        mew_type: MewType::Quote(original_action_hash.clone()),
        content: Some(mew_content.clone()),
    };
    let quote_action_hash = create_entry(EntryTypes::Mew(mew))?;

    let base = get_my_mews_base(MEW_PATH_SEGMENT, true)?;

    let _link_ah = create_link(base, quote_action_hash.clone(), LinkTypes::Mew, ())?;
    // link off original entry as quote
    let _quote_link_ah = create_link(
        original_action_hash.clone(),
        quote_action_hash.clone(),
        LinkTypes::Quote,
        LinkTag::new(QUOTE_PATH_SEGMENT),
    )?;
    parse_mew_text(mew_content, quote_action_hash.clone())?;
    Ok(quote_action_hash.into())
}

// *** Getting mews ***

#[hdk_extern]
pub fn get_mew(action_hash: ActionHash) -> ExternResult<Mew> {
    let element = get(action_hash, GetOptions::default())?.ok_or(wasm_error!(
        WasmErrorInner::Guest(String::from("Mew not found"))
    ))?;

    let mew: Mew =
        element
            .entry()
            .to_app_option()
            .unwrap()
            .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
                "Malformed mew"
            ))))?;

    Ok(mew)
}

#[hdk_extern]
pub fn get_feed_mew_and_context(action_hash: ActionHash) -> ExternResult<FeedMew> {
    let element = get(action_hash.clone(), GetOptions::default())?.ok_or(wasm_error!(
        WasmErrorInner::Guest(String::from("Mew not found"))
    ))?;
    let mew: Mew =
        element
            .entry()
            .to_app_option()
            .unwrap()
            .ok_or(wasm_error!(WasmErrorInner::Guest(String::from(
                "Malformed mew"
            ))))?;

    let lick_links = get_links(
        action_hash.clone(),
        LinkTypes::Lick,
        Some(LinkTag::new(LICK_PATH_SEGMENT)),
    )?;
    let licks: Vec<AgentPubKey> = lick_links
        .into_iter()
        .map(|link| AgentPubKey::from(EntryHash::from(link.target)).into())
        .collect();

    let reply_links = get_links(
        action_hash.clone(),
        LinkTypes::Reply,
        Some(LinkTag::new(REPLY_PATH_SEGMENT)),
    )?;
    let replies: Vec<AnyLinkableHash> = reply_links
        .into_iter()
        .map(|link| link.target.into())
        .collect();

    let mewmew_links = get_links(
        action_hash.clone(),
        LinkTypes::Mewmew,
        Some(LinkTag::new(MEWMEW_PATH_SEGMENT)),
    )?;
    let mewmews: Vec<AnyLinkableHash> = mewmew_links
        .into_iter()
        .map(|mewmew| mewmew.target.into())
        .collect();

    let quote_links = get_links(
        action_hash.clone(),
        LinkTypes::Quote,
        Some(LinkTag::new(QUOTE_PATH_SEGMENT)),
    )?;
    let quotes: Vec<AnyLinkableHash> = quote_links
        .into_iter()
        .map(|link| link.target.into())
        .collect();

    let feed_mew_and_context = FeedMew {
        mew,
        action: element.action().clone(),
        action_hash: element.signed_action().as_hash().clone(),
        replies,
        quotes,
        licks,
        mewmews,
    };
    Ok(feed_mew_and_context)
}

// *** Mews feeds ***

#[hdk_extern]
pub fn mews_by(agent: AgentPubKey) -> ExternResult<Vec<FeedMew>> {
    let base = get_mews_base(agent, MEW_PATH_SEGMENT, false)?;
    let links = get_links(base, LinkTypes::Mew, None)?;

    let mut feed: Vec<FeedMew> = links
        .into_iter()
        .map(|link| get_feed_mew_and_context(ActionHash::from(link.target).into()))
        .filter_map(Result::ok)
        .collect();
    feed.sort_by(|a, b| b.action.timestamp().cmp(&a.action.timestamp()));

    Ok(feed)
}

#[hdk_extern]
pub fn mews_feed(_options: FeedOptions) -> ExternResult<Vec<FeedMew>> {
    let mut feed = Vec::new();
    let me = agent_info()?.agent_latest_pubkey;
    feed.append(&mut mews_by(me)?);
    for agent in my_following(())?.into_iter() {
        feed.append(&mut mews_by(agent)?);
    }
    // TODO don't really need to sort, could merge for efficiency
    // sort by timestamp in descending order
    feed.sort_by(|a, b| b.action.timestamp().cmp(&a.action.timestamp()));

    Ok(feed)
}

#[hdk_extern]
pub fn recommended() -> ExternResult<Vec<FeedMew>> {
    let mut feed = Vec::new();
    let me = agent_info()?.agent_latest_pubkey;
    // TODO figure out how to manage what call returns
    let mut atoms: Vec<TrustAtom> = call(
        CallTargetCell::Local,
        ZomeName::from("trust_atom"),
        FunctionName::from("query_mine"),
        None,
        QueryMineInput {
            target: None,
            content_full: None,
            content_starts_with: None,
            value_starts_with: None,
        },
    )?;

    // filter for those TrustAtoms above a weight threshold (nominally >= 0)
    feed = atoms
        .into_iter()
        .filter_map( |atom|
            match(atom.value) {
                Some(value_string) => {
                    let value_decimal = Decimal::from_str(value_str.as_str());
                    match (value_decimal) {
                        Ok(value_decimal) => {
                            if value_decimal >= Decimal::ZERO{
                                Some(value_string)
                            } else {
                                None
                            }
                        },
                        Err(error) => Err(wasm_error!(
                            "Unexpected value: `{}`.  Error: `{}`",
                            value_string,
                            error
                        )),
                    }
                },
                None => Some(None), // trust atoms with no value typically mean "yup" without a specific "percent/weight"
            }
        ).flat_map( |atom|
            followed_author = atom.target ;
            links_to_posts = get_links(followed_author, LinkTypes::Mew); // TODO we want to get only links for this hashtag: atom.content
            // TODO should we get other types: mewmews, quotes, etc
            links_to_posts.map(|link_to_post|
                get_feed_mew_and_context(ActionHash::from(link_to_post.target).into())
            )

            // TODO: also get the creation date, either from the entry or the link itself

        ).collect();
        feed.sort_by(|a, b| b.action.timestamp().cmp(&a.action.timestamp()));
        // TODO *maybe* also  order these posts according to some combination of: weight of tags and recency
        // (don't want to see very old highly weighted at top)
        // *buttttt* maybe just chron is good!  if only showing last time chunk, or (later) mews since last visit
        // it really depends if that last chunk is 100 mews (chron fine) or 10,000 mews (then weight of tags is awesome)

        Ok(feed)
}

// *** Liking ***

#[hdk_extern]
pub fn lick_mew(action_hash: ActionHash) -> ExternResult<()> {
    let me: AgentPubKey = agent_info()?.agent_latest_pubkey;

    let base = get_my_mews_base(LICK_PATH_SEGMENT, true)?;
    let _my_lick_ah = create_link(base, action_hash.clone(), LinkTypes::Lick, ())?;
    let _mew_lick_ah = create_link(
        action_hash.clone(),
        me.clone(),
        LinkTypes::Lick,
        LinkTag::new(LICK_PATH_SEGMENT),
    )?;
    Ok(())
}

#[hdk_extern]
pub fn my_licks(_: ()) -> ExternResult<Vec<AnyLinkableHash>> {
    let me = agent_info()?.agent_latest_pubkey;
    licks_inner(me, LICK_PATH_SEGMENT)
}

// get mews licked by an agent
#[hdk_extern]
pub fn licks(agent: AgentPubKey) -> ExternResult<Vec<AnyLinkableHash>> {
    licks_inner(agent, LICK_PATH_SEGMENT)
}

fn licks_inner(agent: AgentPubKey, base_type: &str) -> ExternResult<Vec<AnyLinkableHash>> {
    let base = get_mews_base(agent, base_type, false)?;
    let links = get_links(base, LinkTypes::Lick, None)?;
    Ok(links.into_iter().map(|link| link.target).collect())
}

#[hdk_extern]
pub fn unlick_mew(action_hash: ActionHash) -> ExternResult<()> {
    let me: EntryHash = agent_info()?.agent_latest_pubkey.into();
    let mew_licks = get_links(action_hash.clone(), LinkTypes::Lick, None)?;
    for lick in mew_licks {
        if lick.target == me.clone().into() {
            let _delete_link_ah = delete_link(lick.create_link_hash)?;
            break;
        }
    }

    let my_licks = get_my_mews_base(LICK_PATH_SEGMENT, true)?;
    let links = get_links(my_licks.clone(), LinkTypes::Lick, None)?;
    for link in links {
        if link.target == action_hash.clone().into() {
            let _deleted_link_ah = delete_link(link.create_link_hash)?;
            break;
        }
    }
    Ok(())
}

// *** Following ***

#[hdk_extern]
pub fn follow(agent: AgentPubKey) -> ExternResult<()> {
    let me_target: EntryHash = agent_info()?.agent_latest_pubkey.into();
    let them_target: EntryHash = AgentPubKey::from(agent.clone()).into();

    if me_target == them_target {
        return Err(wasm_error!(WasmErrorInner::Guest(String::from(
            "Cannot follow yourself."
        ))));
    }

    let me = get_my_mews_base(FOLLOWING_PATH_SEGMENT, true)?;
    let _following_link_ah = create_link(me, them_target, LinkTypes::Follow, ())?;

    let them = get_mews_base(agent, FOLLOWER_PATH_SEGMENT, true)?;
    let _follower_link_ah = create_link(them, me_target, LinkTypes::Follow, ())?;
    Ok(())
}

#[hdk_extern]
pub fn unfollow(agent: AgentPubKey) -> ExternResult<()> {
    let them_target: EntryHash = AgentPubKey::from(agent.clone()).into();
    let me = get_my_mews_base(FOLLOWING_PATH_SEGMENT, true)?;
    let links = get_links(me.clone(), LinkTypes::Follow, None)?;
    for link in links {
        if link.target == them_target.clone().into() {
            let _deleted_link_ah = delete_link(link.create_link_hash)?;
            break;
        }
    }

    let me_target: EntryHash = agent_info()?.agent_latest_pubkey.into();
    let them = get_mews_base(agent, FOLLOWER_PATH_SEGMENT, true)?;
    let links = get_links(them.clone(), LinkTypes::Follow, None)?;
    for link in links {
        if link.target == me_target.clone().into() {
            let _deleted_link_ah = delete_link(link.create_link_hash)?;
            break;
        }
    }

    Ok(())
}

#[hdk_extern]
pub fn my_following(_: ()) -> ExternResult<Vec<AgentPubKey>> {
    let me = agent_info()?.agent_latest_pubkey;
    follow_inner(me, FOLLOWING_PATH_SEGMENT)
}

#[hdk_extern]
pub fn my_followers(_: ()) -> ExternResult<Vec<AgentPubKey>> {
    let me = agent_info()?.agent_latest_pubkey;
    follow_inner(me, FOLLOWER_PATH_SEGMENT)
}

// get who an agent is following
#[hdk_extern]
pub fn following(agent: AgentPubKey) -> ExternResult<Vec<AgentPubKey>> {
    follow_inner(agent, FOLLOWING_PATH_SEGMENT)
}

// get followers of an agent
#[hdk_extern]
pub fn followers(agent: AgentPubKey) -> ExternResult<Vec<AgentPubKey>> {
    follow_inner(agent, FOLLOWER_PATH_SEGMENT)
}

fn follow_inner(agent: AgentPubKey, base_type: &str) -> ExternResult<Vec<AgentPubKey>> {
    let base = get_mews_base(agent, base_type, false)?;
    let links = get_links(base, LinkTypes::Follow, None)?;
    Ok(links
        .into_iter()
        .map(|link| AgentPubKey::from(EntryHash::from(link.target)).into())
        .collect())
}

// *** Tagging (hashtag, cashtag, mention) ***

#[hdk_extern]
pub fn get_mews_with_hashtag(hashtag: String) -> ExternResult<Vec<FeedMew>> {
    let path = Path::from(format!("hashtags.{}", hashtag));
    Ok(get_mews_from_path(path)?)
}

#[hdk_extern]
pub fn get_mews_with_cashtag(cashtag: String) -> ExternResult<Vec<FeedMew>> {
    let path = Path::from(format!("cashtags.{}", cashtag));
    Ok(get_mews_from_path(path)?)
}

#[hdk_extern]
pub fn get_mews_with_mention(agent_pub_key: AgentPubKey) -> ExternResult<Vec<FeedMew>> {
    let path = Path::from(format!("mentions.{}", agent_pub_key));
    Ok(get_mews_from_path(path)?)
}

pub fn get_mews_from_path(path: Path) -> ExternResult<Vec<FeedMew>> {
    let path_hash = path.path_entry_hash()?;

    let links = get_links(path_hash, LinkTypes::Tag, None)?;
    let mut mews: Vec<FeedMew> = links
        .into_iter()
        .map(|link| {
            get(ActionHash::from(link.target), GetOptions::default())
                .unwrap()
                .unwrap()
        })
        .map(|element| get_feed_mew_and_context(element.signed_action().as_hash().clone()).unwrap())
        .collect();
    mews.sort_by(|a, b| b.action.timestamp().cmp(&a.action.timestamp()));
    Ok(mews)
}

pub fn parse_mew_text(mew_content: MewContent, mew_hash: ActionHash) -> ExternResult<()> {
    let hashtag_regex = Regex::new(r"#\w+").unwrap();
    let cashtag_regex = Regex::new(r"\$\w+").unwrap();
    for mat in hashtag_regex.find_iter(&mew_content.text.clone()) {
        create_mew_tag_links("hashtags", mat.as_str(), mew_hash.clone())?;
    }
    for mat in cashtag_regex.find_iter(&mew_content.text.clone()) {
       create_mew_tag_links("cashtags", mat.as_str(), mew_hash.clone())?;
    }
    if let Some(links) = mew_content.links {
        for link in links {
            match link {
                LinkTarget::Mention(mention) => {
                    let path = Path::from(format!("mentions.{}", mention));
                    let path_hash = path.path_entry_hash()?;
                    let _link_ah = create_link(path_hash, mew_hash.clone(), LinkTypes::Tag, ())?;
                }
                _ => ()
            }
        }
    }
    Ok(())
}

#[hdk_extern]
pub fn search_hashtags(query: String) -> ExternResult<Vec<String>> {
    search_tags("hashtags".into(), query)
}

#[hdk_extern]
pub fn search_cashtags(query: String) -> ExternResult<Vec<String>> {
    search_tags("cashtags".into(), query)
}

fn search_tags(path_stem: String, content: String) -> ExternResult<Vec<String>> {
    let prefix: String = content.chars().take(3).collect();
    let path = Path::from(format!("search_{}.{}", path_stem, prefix));

    let links = get_links(path.path_entry_hash()?, LinkTypes::TagPrefix, None)?;

    let tags: Vec<String> = links
        .into_iter()
        .map(|link|
            String::from_utf8(link.tag.into_inner()).map_err(|_| wasm_error!(WasmErrorInner::Guest("Failed to convert link tag to string".into())))

        )
        .filter_map(Result::ok)
        .collect();

    Ok(tags)
}

fn create_mew_tag_links(path_stem: &str, content: &str, mew_hash: ActionHash) -> ExternResult<()> {
    // Link from Path cashtags.mytag -> mew
    let path = Path::from(format!("{}.{}", path_stem, content));
    let path_hash = path.path_entry_hash()?;
    path.typed(LinkTypes::Tag)?.ensure()?;
    let _link_ah = create_link(path_hash.clone(), mew_hash, LinkTypes::Tag, ())?;

    // Create Path for sliced hashtag (first 3 characters) under hashtags_search.myt
    // Link from Path hashtags.myt -> hashtags.mytag Path
    let word: String =  content.chars().skip(1).collect();
    let word_lowercase = word.to_lowercase();
    let prefix: String = word_lowercase.chars().take(3).collect();

    let search_path = Path::from(format!("search_{}.{}", path_stem, prefix));
    let search_path_hash = search_path.path_entry_hash()?;
    search_path.typed(LinkTypes::TagPrefix)?.ensure()?;
    let _link_search_tag = create_link(search_path_hash, path_hash.clone(), LinkTypes::TagPrefix, word.as_bytes().to_vec())?;

    Ok(())
}
