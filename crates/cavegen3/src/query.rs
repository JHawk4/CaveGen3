//! The query language for seed searches, e.g. `sh6 bluekochappy/bey_goma carry dist < 300`.
//!
//! Ported from Caveripper's `query` module (MIT, https://github.com/mayabyte/caveripper), using
//! the same grammar. See QUERY.md in Caveripper for the language.

mod search;
pub mod special;

use std::{cmp::Ordering, fmt::Display};

use pest::{
    Parser,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;
pub use search::find_matching_layouts_parallel;

use crate::{
    caveinfo::{CapInfo, CaveUnit, RoomType, TekiInfo},
    data::{GameData, Sublevel},
    error::{Error, Result},
    layout::{Layout, SpawnObject},
    point::Point,
    rng::Generator,
};

#[derive(Parser)]
#[grammar = "query/grammar.pest"]
struct QueryParser;

/// A condition on the layouts a seed generates.
pub trait Query {
    /// Whether the layouts `seed` generates with `generator`'s `rand()` match.
    fn matches_with(&self, generator: Generator, seed: u64, data: &GameData) -> bool;

    /// [`Query::matches_with`] for the GameCube generator.
    fn matches(&self, seed: u32, data: &GameData) -> bool {
        self.matches_with(Generator::Gc, seed as u64, data)
    }
}

/// One or more conditions joined with `&`, each on a sublevel.
#[derive(Clone, Debug)]
pub struct StructuralQuery {
    /// Each distinct sublevel the clauses mention, generated once per seed.
    pub sublevels: Vec<Sublevel>,
    pub clauses: Vec<QueryClause>,
}

impl Query for StructuralQuery {
    fn matches_with(&self, generator: Generator, seed: u64, data: &GameData) -> bool {
        let layouts: Vec<Layout> = self
            .sublevels
            .iter()
            .map(|sublevel| Layout::generate_with(generator, seed, data.caveinfo(sublevel).expect("sublevel was checked when parsing")))
            .collect();
        self.clauses.iter().all(|clause| clause.querykind.matches(&layouts[clause.sublevel]))
    }
}

impl StructuralQuery {
    /// Parses a query such as `scx7 minihoudai < 2 & sh6 hole carry dist < 1000`. Sublevel names
    /// without a `game:` prefix are vanilla.
    pub fn try_parse(input: &str, data: &GameData) -> Result<Self> {
        Self::try_parse_in(input, data, None)
    }

    /// Like [`StructuralQuery::try_parse`], but sublevel names without a `game:` prefix belong to
    /// `default_game` when one is given.
    pub fn try_parse_in(input: &str, data: &GameData, default_game: Option<&str>) -> Result<Self> {
        let pairs = QueryParser::parse(Rule::query, input).map_err(|e| Error::new(format!("couldn't parse query: {e}")))?;
        let mut sublevels: Vec<Sublevel> = Vec::new();
        let mut current: Option<usize> = None;
        let mut clauses = Vec::new();
        for pair in pairs {
            match pair.as_rule() {
                Rule::sublevel_ident => {
                    let sublevel = data.sublevel_in(pair.as_str(), default_game)?;
                    data.caveinfo(&sublevel)?;
                    current = Some(sublevels.iter().position(|s| *s == sublevel).unwrap_or_else(|| {
                        sublevels.push(sublevel);
                        sublevels.len() - 1
                    }));
                }
                Rule::expression => {
                    let sublevel = current.ok_or_else(|| Error::new("query clause has no sublevel"))?;
                    clauses.push(QueryClause { sublevel, querykind: QueryKind::try_parse(pair, data)? });
                }
                Rule::EOI => {}
                rule => return Err(Error::new(format!("unexpected {rule:?} in query"))),
            }
        }
        Ok(StructuralQuery { sublevels, clauses })
    }
}

impl Display for StructuralQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, clause) in self.clauses.iter().enumerate() {
            if i > 0 {
                write!(f, " & ")?;
            }
            write!(f, "{} {}", self.sublevels[clause.sublevel].short_name(), clause.querykind)?;
        }
        Ok(())
    }
}

/// A condition on one sublevel: an index into [`StructuralQuery::sublevels`] and what to check.
#[derive(Clone, Debug)]
pub struct QueryClause {
    pub sublevel: usize,
    pub querykind: QueryKind,
}

#[derive(Clone, Debug)]
pub enum QueryKind {
    CountEntity { entity_matcher: EntityMatcher, relationship: Ordering, amount: usize },
    CountRoom { unit_matcher: UnitMatcher, relationship: Ordering, amount: usize },
    CarryDist { entity: EntityMatcher, relationship: Ordering, req_dist: f32 },
    StraightLineDist { entity1: EntityMatcher, entity2: EntityMatcher, relationship: Ordering, req_dist: f32 },
    Gated(EntityMatcher),
    NotGated(EntityMatcher),
    RoomPath(RoomPath),
}

fn compares(value: f32, relationship: Ordering, target: f32) -> bool {
    value.partial_cmp(&target) == Some(relationship)
}

/// Total distance along a carry path, in order.
fn path_length(points: impl Iterator<Item = Point>) -> f32 {
    let mut total = 0.0;
    let mut prev: Option<Point> = None;
    for p in points {
        if let Some(q) = prev {
            total += q.dist(&p);
        }
        prev = Some(p);
    }
    total
}

/// Consecutive pairs of points along a path.
fn path_segments(points: impl Iterator<Item = Point>) -> Vec<(Point, Point)> {
    let points: Vec<Point> = points.collect();
    points.windows(2).map(|w| (w[0], w[1])).collect()
}

impl QueryKind {
    /// Whether the layout satisfies this condition.
    pub fn matches(&self, layout: &Layout) -> bool {
        match self {
            QueryKind::CountEntity { entity_matcher, relationship, amount } => {
                layout.spawn_objects().filter(|(so, _)| entity_matcher.matches(so)).count().cmp(amount) == *relationship
            }
            QueryKind::CountRoom { unit_matcher, relationship, amount } => {
                layout.units.iter().filter(|u| unit_matcher.matches(u.unit)).count().cmp(amount) == *relationship
            }
            QueryKind::CarryDist { entity, relationship, req_dist } => layout
                .spawn_objects()
                .filter(|(so, _)| entity.matches(so))
                .map(|(_, pos)| path_length(layout.waypoint_graph().carry_path_wps_pos(pos)))
                .any(|d| compares(d, *relationship, *req_dist)),
            QueryKind::StraightLineDist { entity1, entity2, relationship, req_dist } => {
                let e2s: Vec<Point> = layout.spawn_objects().filter(|(so, _)| entity2.matches(so)).map(|(_, p)| p).collect();
                layout
                    .spawn_objects()
                    .filter(|(so, _)| entity1.matches(so))
                    .any(|(_, p1)| e2s.iter().any(|p2| compares(p1.p2_dist(p2), *relationship, *req_dist)))
            }
            QueryKind::Gated(entity_matcher) => {
                let gates = gate_positions(layout);
                layout.spawn_objects().filter(|(so, _)| entity_matcher.matches(so)).any(|(_, pos)| {
                    path_segments(layout.waypoint_graph().carry_path_wps_pos(pos))
                        .into_iter()
                        .any(|(p1, p2)| gates.iter().any(|g| point_to_line_dist(*g, p1, p2) < 80.0))
                })
            }
            QueryKind::NotGated(entity_matcher) => {
                let gates = gate_positions(layout);
                layout.spawn_objects().filter(|(so, _)| entity_matcher.matches(so)).all(|(_, pos)| {
                    path_segments(layout.waypoint_graph().carry_path_wps_pos(pos))
                        .into_iter()
                        .all(|(p1, p2)| gates.iter().all(|g| point_to_line_dist(*g, p1, p2) > 80.0))
                })
            }
            QueryKind::RoomPath(path) => path.matches(layout),
        }
    }

    fn try_parse(input: Pair<'_, Rule>, data: &GameData) -> Result<Self> {
        let full = input.as_str().to_string();
        let bad = || Error::new(format!("couldn't understand \"{full}\""));
        let expr = input.into_inner().next().ok_or_else(bad)?;
        match (expr.as_rule(), expr.into_inner()) {
            (Rule::compare, inner) => {
                let values: Vec<&str> = inner.map(|v| v.as_str().trim()).collect();
                let bare = values[0].split('/').next().unwrap_or(values[0]);
                let bare_lower = bare.to_ascii_lowercase();
                let amount = values[2].parse::<usize>().map_err(|_| bad())?;
                let relationship = char_to_ordering(values[1]);
                let catalog = data.catalog();
                if catalog.teki.contains(&bare_lower)
                    || catalog.treasures.iter().any(|t| t.internal_name.eq_ignore_ascii_case(bare))
                    || ["hole", "geyser", "ship", "gate"].contains(&bare_lower.as_str())
                {
                    Ok(QueryKind::CountEntity { entity_matcher: values[0].into(), relationship, amount })
                } else if catalog.units.contains(&bare_lower) || RoomType::parse(values[0]).is_some() {
                    Ok(QueryKind::CountRoom { unit_matcher: values[0].into(), relationship, amount })
                } else {
                    Err(Error::new(format!("\"{}\" isn't a known teki, treasure or map unit", values[0])))
                }
            }
            (Rule::carry_dist, inner) => {
                let values: Vec<&str> = inner.map(|v| v.as_str()).collect();
                Ok(QueryKind::CarryDist {
                    entity: values[0].into(),
                    relationship: char_to_ordering(values[1]),
                    req_dist: values[2].parse().map_err(|_| bad())?,
                })
            }
            (Rule::straight_dist, inner) => {
                let values: Vec<&str> = inner.map(|v| v.as_str()).collect();
                Ok(QueryKind::StraightLineDist {
                    entity1: values[0].into(),
                    entity2: values[1].into(),
                    relationship: char_to_ordering(values[2]),
                    req_dist: values[3].parse().map_err(|_| bad())?,
                })
            }
            (Rule::gated, inner) => Ok(QueryKind::Gated(inner.as_str().into())),
            (Rule::not_gated, inner) => Ok(QueryKind::NotGated(inner.as_str().into())),
            (Rule::room_path, inner) => Ok(QueryKind::RoomPath(inner.into())),
            _ => Err(bad()),
        }
    }
}

fn gate_positions(layout: &Layout) -> Vec<Point> {
    layout.spawn_objects().filter(|(so, _)| matches!(so, SpawnObject::Gate(..))).map(|(_, p)| p).collect()
}

fn ordering_char(o: &Ordering) -> char {
    match o {
        Ordering::Less => '<',
        Ordering::Equal => '=',
        Ordering::Greater => '>',
    }
}

impl Display for QueryKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryKind::CountEntity { entity_matcher, relationship, amount } => write!(f, "{entity_matcher} {} {amount}", ordering_char(relationship)),
            QueryKind::CountRoom { unit_matcher, relationship, amount } => write!(f, "{unit_matcher} {} {amount}", ordering_char(relationship)),
            QueryKind::CarryDist { entity, relationship, req_dist } => write!(f, "{entity} carry dist {} {req_dist}", ordering_char(relationship)),
            QueryKind::StraightLineDist { entity1, entity2, relationship, req_dist } => {
                write!(f, "{entity1} straight dist {entity2} {} {req_dist}", ordering_char(relationship))
            }
            QueryKind::Gated(entity) => write!(f, "{entity} gated"),
            QueryKind::NotGated(entity) => write!(f, "{entity} not gated"),
            QueryKind::RoomPath(path) => {
                for (i, (unit_matcher, entity_matchers)) in path.components.iter().enumerate() {
                    if i > 0 {
                        write!(f, " -> ")?;
                    }
                    write!(f, "{unit_matcher}")?;
                    for em in entity_matchers {
                        write!(f, " + {em}")?;
                    }
                }
                Ok(())
            }
        }
    }
}

/// A chain of connected map units, each optionally containing certain objects.
#[derive(Debug, Clone)]
pub struct RoomPath {
    components: Vec<(UnitMatcher, Vec<EntityMatcher>)>,
}

impl RoomPath {
    fn matches(&self, layout: &Layout) -> bool {
        (0..layout.units.len()).any(|start| {
            let mut frontier = vec![start];
            let mut visited: Vec<usize> = Vec::new();
            for (unit_matcher, entity_matchers) in self.components.iter() {
                if frontier.is_empty() {
                    return false;
                }
                let mut new_frontier = Vec::new();
                let mut matched = false;
                for &unit in frontier.iter() {
                    if visited.contains(&unit) {
                        continue;
                    }
                    visited.push(unit);
                    let contents = || layout.unit_spawnpoints(unit).iter().flat_map(|sp| sp.contains.iter());
                    if unit_matcher.matches(layout.units[unit].unit) && entity_matchers.iter().all(|em| contents().any(|so| em.matches(so))) {
                        matched = true;
                        new_frontier.extend(
                            layout
                                .unit_doors(unit)
                                .iter()
                                .map(|d| layout.doors[d.adjacent.expect("door isn't attached") as usize].unit as usize)
                                .filter(|&n| n != unit),
                        );
                    }
                }
                if !matched {
                    return false;
                }
                frontier = new_frontier;
            }
            true
        })
    }
}

impl From<Pairs<'_, Rule>> for RoomPath {
    fn from(input: Pairs<'_, Rule>) -> Self {
        let components = input
            .map(|component| {
                let mut pairs = component.into_inner();
                (pairs.next().unwrap().as_str().into(), pairs.map(|e| e.as_str().into()).collect())
            })
            .collect();
        RoomPath { components }
    }
}

/// Matches an object by name (teki, cap teki or treasure; "any" matches all), optionally
/// with what it carries (`name/treasure`), or the hole, geyser, ship or a gate.
#[derive(Debug, Clone)]
pub enum EntityMatcher {
    Entity { name: String, carrying: Option<String> },
    Hole,
    Geyser,
    Ship,
    Gate,
}

impl EntityMatcher {
    pub fn matches(&self, spawn_object: &SpawnObject) -> bool {
        match (self, spawn_object) {
            (
                EntityMatcher::Entity { name, carrying },
                SpawnObject::Teki(TekiInfo { internal_name, carrying: i_carrying, .. }, _)
                | SpawnObject::CapTeki(CapInfo { internal_name, carrying: i_carrying, .. }, _),
            ) => {
                let name_matches = name.eq_ignore_ascii_case("any") || name.eq_ignore_ascii_case(internal_name);
                let carrying_matches =
                    carrying.as_ref().is_none_or(|c1| i_carrying.as_ref().map_or(c1.eq_ignore_ascii_case("any"), |c2| c1.eq_ignore_ascii_case(c2)));
                name_matches && carrying_matches
            }
            (EntityMatcher::Entity { name, carrying }, SpawnObject::Item(item)) => {
                (name.eq_ignore_ascii_case("any") || name.eq_ignore_ascii_case(&item.internal_name)) && carrying.is_none()
            }
            (EntityMatcher::Hole, SpawnObject::Hole(_))
            | (EntityMatcher::Geyser, SpawnObject::Geyser(_))
            | (EntityMatcher::Ship, SpawnObject::Ship)
            | (EntityMatcher::Gate, SpawnObject::Gate(..)) => true,
            _ => false,
        }
    }
}

impl From<&str> for EntityMatcher {
    fn from(s: &str) -> Self {
        match s.to_ascii_lowercase().trim() {
            "hole" => EntityMatcher::Hole,
            "geyser" => EntityMatcher::Geyser,
            "ship" => EntityMatcher::Ship,
            "gate" => EntityMatcher::Gate,
            s => match s.split_once('/') {
                Some((name, carrying)) => EntityMatcher::Entity { name: name.trim().to_string(), carrying: Some(carrying.trim().to_string()) },
                None => EntityMatcher::Entity { name: s.to_string(), carrying: None },
            },
        }
    }
}

impl Display for EntityMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntityMatcher::Hole => write!(f, "hole"),
            EntityMatcher::Geyser => write!(f, "geyser"),
            EntityMatcher::Ship => write!(f, "ship"),
            EntityMatcher::Gate => write!(f, "gate"),
            EntityMatcher::Entity { name, carrying: None } => write!(f, "{name}"),
            EntityMatcher::Entity { name, carrying: Some(carrying) } => write!(f, "{name}/{carrying}"),
        }
    }
}

/// Matches map units by type ("room", "cap"/"alcove", "hall"/"hallway"), name, or "any".
#[derive(Debug, Clone)]
pub enum UnitMatcher {
    UnitType(RoomType),
    Named(String),
}

impl UnitMatcher {
    pub fn matches(&self, unit: &CaveUnit) -> bool {
        match self {
            UnitMatcher::UnitType(t) => unit.room_type == *t,
            UnitMatcher::Named(name) => name.eq_ignore_ascii_case("any") || unit.unit_folder_name.eq_ignore_ascii_case(name),
        }
    }
}

impl From<&str> for UnitMatcher {
    fn from(input: &str) -> Self {
        match RoomType::parse(input) {
            Some(t) => UnitMatcher::UnitType(t),
            None => UnitMatcher::Named(input.to_string()),
        }
    }
}

impl Display for UnitMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitMatcher::UnitType(t) => write!(f, "{t}"),
            UnitMatcher::Named(name) if name.eq_ignore_ascii_case("any") => write!(f, "any(room)"),
            UnitMatcher::Named(name) => write!(f, "{name}"),
        }
    }
}

fn char_to_ordering(c: &str) -> Ordering {
    match c {
        "<" => Ordering::Less,
        ">" => Ordering::Greater,
        _ => Ordering::Equal,
    }
}

fn point_to_line_dist(p: Point, l1: Point, l2: Point) -> f32 {
    let len = l1.dist(&l2);
    if len <= 0.0 {
        return f32::MAX;
    }
    let norm = (l1 - l2).normalized();
    let t = norm.dot(p - l1) / len;
    if t <= 0.0 {
        p.dist(&l1)
    } else if t >= 1.0 {
        p.dist(&l2)
    } else {
        ((norm * len * t) + l1 - p).length()
    }
}
