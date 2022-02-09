use rusqlite::named_params;
use rusqlite::Connection;
use serde::Deserialize;
use serde::Serialize;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug, Default)]
pub(crate) struct TagRule {
    pub(crate) id: u32,
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_tags_ids")]
    pub(crate) tags_ids: Vec<u32>,
    pub(crate) matching_pattern: String,
    pub(crate) is_regex: bool,
}

pub fn deserialize_tags_ids<'de, D>(deserializer: D) -> Result<Vec<u32>, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    #[derive(Debug)]
    struct StringVecVisitor(PhantomData<Vec<u32>>);

    impl<'de> serde::de::Visitor<'de> for StringVecVisitor {
        type Value = Vec<u32>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            dbg!(&self);
            formatter.write_str("a string containing a list of integer IDs separated by commas.")
        }


        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let mut ids = Vec::new();
            for id in v.split(",") {
                let id = id.parse::<u32>().unwrap_or(0);
                if id != 0 {
                    ids.push(id);
                }
            }
            Ok(ids)
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
            where E: serde::de::Error
        {
            Ok(Vec::new())
        }

        fn visit_seq<S>(self, visitor: S) -> Result<Self::Value, S::Error>
            where S: serde::de::SeqAccess<'de>
        {
            Deserialize::deserialize(serde::de::value::SeqAccessDeserializer::new(visitor))
        }
    }

    deserializer.deserialize_any(StringVecVisitor(PhantomData))
}

pub(crate) fn find_all(conn: &Connection) -> Vec<TagRule> {
    let mut stmt = conn
        .prepare(
            "
        SELECT id,
        matching_pattern,
        is_regex,
        (
            SELECT GROUP_CONCAT(tag_id)
            FROM tag_rule_tag
            WHERE tag_rule_id = tag_rules.id
        ) AS tags_ids
        FROM tag_rules
    ",
        )
        .expect("Could not fetch tag_rules");

    let mut tag_rules: Vec<TagRule> = Vec::new();

    let mut rows_iter = serde_rusqlite::from_rows::<TagRule>(stmt.query([]).unwrap());

    loop {
        match rows_iter.next() {
            None => {
                break;
            }
            Some(tag_rule) => {
                let tag_rule = tag_rule.expect("Could not deserialize TagRule item");
                tag_rules.push(tag_rule);
            }
        }
    }

    tag_rules
}

pub(crate) fn update(conn: &Connection, tag_rule: TagRule) {

    // Update tag rule
    let mut stmt = conn
        .prepare(
            "
        UPDATE tag_rules
        SET is_regex = :is_regex,
            matching_pattern = :matching_pattern
        WHERE id = :id
    ",
        )
        .unwrap();

    stmt.execute(named_params! {
        ":id": &tag_rule.id,
        ":is_regex": &tag_rule.is_regex,
        ":matching_pattern": &tag_rule.matching_pattern,
    })
    .expect("Could not update tag");

    // Remove previous tags associations
    let mut stmt = conn
        .prepare("DELETE FROM tag_rule_tag WHERE tag_id = :id")
        .expect("Could not create query to delete tag rule associations")
    ;

    stmt.execute(named_params! {":id": &tag_rule.id}).expect("Could not delete tag rule associations");

    let mut stmt = conn
        .prepare(
            "
        INSERT INTO tag_rule_tag (
            tag_rule_id,
            tag_id
        )
        VALUES (
            :tag_rule_id,
            :tag_id
        )
        ",
        )
        .expect("Could not create query to insert tag rule with tag association");

    for tags_id in tag_rule.tags_ids {
        stmt.execute(named_params! {
            ":tag_rule_id": &tag_rule.id,
            ":tag_id": &tags_id,
        })
            .expect("Could not insert tag rule with tag associations");
    }
}

pub(crate) fn apply_rules(conn: &Connection) -> usize {
    println!("TODO: apply tag rules");

    0
}

pub(crate) fn create(conn: &Connection, tag_rule: TagRule) -> usize {
    let mut stmt = conn
        .prepare(
            "
            INSERT INTO tag_rules (
                is_regex,
                matching_pattern
            )
            VALUES (
                :is_regex,
                :matching_pattern
            )
        ",
        )
        .expect("Could not create query to create new tag rule");

    stmt.execute(named_params! {
        ":is_regex": &tag_rule.is_regex,
        ":matching_pattern": &tag_rule.matching_pattern,
    })
    .expect("Could not insert tag");

    let id = conn.last_insert_rowid();

    let mut stmt = conn
        .prepare(
            "
        INSERT INTO tag_rule_tag (
            tag_rule_id,
            tag_id
        )
        VALUES (
            :tag_rule_id,
            :tag_id
        )
        ",
        )
        .expect("Could not create query to insert tag rule with tag association");

    for tags_id in tag_rule.tags_ids {
        stmt.execute(named_params! {
            ":tag_rule_id": &id,
            ":tag_id": &tags_id,
        })
        .expect("Could not insert tag rule with tag association");
    }

    id as usize
}