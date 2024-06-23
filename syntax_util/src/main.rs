use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
    iter::once,
};

use itertools::Itertools;

macro_rules! syntax_def {
    (@rule $left:ident => $($right:expr)+) => {{
        #[allow(unused_imports)]
        use $crate::NonTerminal::*;

        ($crate::NonTerminal::$left, vec![$(Symbol::from($right),)+])
    }};

    ($($left:ident => $($right:expr)+),+ $(,)?) => {
        vec![$(syntax_def!(@rule $left => $($right)+)),+]
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Symbol {
    NT(NonTerminal),
    T(&'static str),
}

impl Symbol {
    const EPS: Self = Self::T("");

    fn symbol_name(&self) -> &str {
        match self {
            Self::NT(nt) => nt.symbol_name(),
            Self::T(t) => t,
        }
    }

    #[allow(dead_code)]
    fn is_terminal(&self) -> bool {
        matches!(self, Self::T(_))
    }

    fn to_non_terminal(&self) -> Option<NonTerminal> {
        match self {
            Self::NT(nt) => Some(*nt),
            Self::T(_) => None,
        }
    }

    #[allow(dead_code)]
    fn to_ternimal(&self) -> Option<&str> {
        match self {
            Self::NT(_) => None,
            Self::T(t) => Some(t),
        }
    }
}

impl From<NonTerminal> for Symbol {
    fn from(value: NonTerminal) -> Self {
        Self::NT(value)
    }
}

impl From<&'static str> for Symbol {
    fn from(value: &'static str) -> Self {
        Self::T(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum NonTerminal {
    Start,
    StmtList,
    Stmt,
    IfStmt,
    WhileStmt,
    AssignStmt,
    DefStmt,
    Block,
    Expr,
    LogicalExpr,
    RelExpr,
    SimpleExpr,
    Term,
    Factor,
    IdList,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LR1Item {
    left: NonTerminal,
    right: Vec<Symbol>,
    pos: usize,
    next: String,
}

impl LR1Item {
    fn new(left: NonTerminal, right: &[Symbol], next: String) -> Self {
        Self {
            left,
            right: right.to_vec(),
            pos: 0,
            next,
        }
    }

    fn advanced(&self) -> Self {
        Self {
            left: self.left,
            right: self.right.clone(),
            pos: self.pos + 1,
            next: self.next.clone(),
        }
    }

    fn current_symbol(&self) -> Option<&Symbol> {
        self.right.get(self.pos)
    }

    fn next_ternimal_or<'a>(
        &'a self,
        default: String,
        first_set: &'a HashMap<NonTerminal, HashSet<&str>>,
    ) -> Box<dyn Iterator<Item = String> + '_> {
        self.right.get(self.pos + 1).map_or_else(
            move || Box::new(once(default)) as Box<dyn Iterator<Item = String>>,
            move |x| match x {
                Symbol::NT(nt) => Box::new(first_set[nt].iter().map(|x| x.to_string())),
                Symbol::T(t) => Box::new(once(t.to_string())),
            },
        )
    }

    fn closure<'a>(
        &self,
        syntax: &'a [(NonTerminal, Vec<Symbol>)],
        first_set: &'a HashMap<NonTerminal, HashSet<&str>>,
    ) -> HashSet<Self> {
        closure(self.clone(), |item, set| {
            if let Some(Symbol::NT(nt)) = item.current_symbol() {
                for next in item.next_ternimal_or(self.next.clone(), first_set) {
                    set.extend(
                        syntax
                            .iter()
                            .filter(|&(left, _)| left == nt)
                            .map(|(left, right)| Self::new(*left, right, next.clone())),
                    );
                }
            }
        })
    }
}

impl NonTerminal {
    fn symbol_name(&self) -> &'static str {
        match self {
            Self::Start => "Start",
            Self::StmtList => "StmtList",
            Self::Stmt => "Stmt",
            Self::IfStmt => "IfStmt",
            Self::WhileStmt => "WhileStmt",
            Self::AssignStmt => "AssignStmt",
            Self::DefStmt => "DefStmt",
            Self::Block => "Block",
            Self::Expr => "Expr",
            Self::LogicalExpr => "LogicalExpr",
            Self::RelExpr => "RelExpr",
            Self::SimpleExpr => "SimpleExpr",
            Self::Term => "Term",
            Self::Factor => "Factor",
            Self::IdList => "IdList",
        }
    }
}

// fn stringify(left: NonTerminal, right: &[Symbol]) -> String {
//     format!(
//         "{}>{}",
//         left.symbol_name(),
//         right.iter().map(|x| x.symbol_name()).join(",")
//     )
// }

fn closure<T, F>(init: T, mut f: F) -> HashSet<T>
where
    T: Eq + Hash,
    F: FnMut(&T, &mut HashSet<T>),
{
    let mut result = HashSet::new();
    result.insert(init);
    loop {
        let mut increment = HashSet::new();
        for item in &result {
            f(item, &mut increment);
        }
        if increment.is_subset(&result) {
            break;
        }
        result.extend(increment);
    }
    result
}

fn first_set(syntax: &[(NonTerminal, Vec<Symbol>)]) -> HashMap<NonTerminal, HashSet<&str>> {
    // TODO: consider epsilon

    let mut result: HashMap<_, HashSet<&str>> = HashMap::new();
    let mut deps: HashMap<_, HashSet<NonTerminal>> = HashMap::new();
    let mut left_nts = HashSet::new();
    let mut deps_required_nts = HashSet::new();

    for (left, right) in syntax {
        left_nts.insert(*left);

        let first_left = result.entry(*left).or_default();

        let rfirst = &right[0];
        if rfirst.symbol_name() == left.symbol_name() {
            // A -> A...
            continue;
        }

        if let Some(rfirst_nt) = rfirst.to_non_terminal() {
            // A -> B...
            // FIRST(A) |= FIRST(B)
            deps.entry(*left).or_default().insert(rfirst_nt);
            deps_required_nts.insert(*left);
        } else {
            // A -> a...
            // FIRST(A) |= {a}
            first_left.insert(rfirst.symbol_name());
        }
    }

    let mut nodeps_nts = left_nts
        .difference(&deps_required_nts)
        .copied()
        .collect::<HashSet<_>>();

    assert!(!nodeps_nts.is_empty());

    let mut finished_nts = nodeps_nts.clone();

    while !nodeps_nts.is_empty() {
        let mut new_nodeps = HashSet::new();

        for nt in &nodeps_nts {
            let subset = result.get(nt).unwrap().clone();
            for (super_nt, dep_set) in deps.iter_mut() {
                if dep_set.contains(nt) {
                    dep_set.remove(nt);
                    result.get_mut(super_nt).unwrap().extend(&subset);

                    if dep_set.is_empty() {
                        new_nodeps.insert(*super_nt);
                    }
                }
            }
        }

        finished_nts.extend(&new_nodeps);
        nodeps_nts = new_nodeps;
    }

    assert_eq!(
        finished_nts.len(),
        left_nts.len(),
        "some non-terminals cannot be processed: {}",
        left_nts
            .difference(&finished_nts)
            .map(|x| x.symbol_name())
            .join(", ")
    );

    result
}

fn follow_set<'a>(
    syntax: &'a [(NonTerminal, Vec<Symbol>)],
    start_symbol: NonTerminal,
    first_set: &'a HashMap<NonTerminal, HashSet<&str>>,
) -> HashMap<NonTerminal, HashSet<&'a str>> {
    let mut result: HashMap<_, HashSet<&str>> = HashMap::new();

    result.entry(start_symbol).or_default().insert("[$]");

    loop {
        let mut increased = false;

        for (left, right) in syntax {
            for (current, follow) in right.iter().chain(once(&Symbol::EPS)).tuple_windows() {
                if let Some(current) = current.to_non_terminal() {
                    match follow.to_non_terminal() {
                        Some(follow_nt) => {
                            // A -> ...BC...
                            // FOLLOW(B) |= FIRST(C)
                            result
                                .entry(current)
                                .or_default()
                                .extend(&first_set[&follow_nt]);
                        }
                        None if follow.symbol_name().is_empty() => {
                            // A -> ...B
                            // FOLLOW(B) |= FOLLOW(A)
                            if let Some(follow_left) = result.get(left).cloned() {
                                let follow_current = result.entry(current).or_default();
                                if !follow_current.is_superset(&follow_left) {
                                    follow_current.extend(&follow_left);
                                    increased = true;
                                }
                            }
                        }
                        _ => {
                            // A -> ...Ba...
                            // FOLLOW(B) |= {a}
                            result
                                .entry(current)
                                .or_default()
                                .insert(follow.symbol_name());
                        }
                    }
                }
            }
        }

        if !increased {
            break;
        }
    }

    result
}

fn lr1_item_sets<'a>(
    syntax: &'a [(NonTerminal, Vec<Symbol>)],
    first_set: &'a HashMap<NonTerminal, HashSet<&str>>,
) -> (Vec<HashSet<LR1Item>>, Vec<HashMap<Symbol, usize>>) {
    let mut item_sets = Vec::new();
    let mut next_states = Vec::new();

    let start_rule = &syntax[0];
    item_sets
        .push(LR1Item::new(start_rule.0, &start_rule.1, "[$]".into()).closure(syntax, first_set));
    next_states.push(HashMap::new());
    let mut idx = 0;
    while idx < item_sets.len() {
        let item_set = &item_sets[idx];
        let mut next_state: HashMap<Symbol, HashSet<_>> = HashMap::new();

        for item in item_set {
            if let Some(sym) = item.current_symbol() {
                let next = item.advanced();
                next_state
                    .entry(sym.clone())
                    .or_default()
                    .extend(next.closure(syntax, first_set));
            } else {
                // ACTION[next] = rx/acc
            }
        }

        for (k, v) in next_state {
            let goto = item_sets.iter().position(|x| x == &v).unwrap_or_else(|| {
                item_sets.push(v);
                next_states.push(HashMap::new());
                item_sets.len() - 1
            });
            next_states[idx].insert(k, goto);
        }

        idx += 1;
    }

    (item_sets, next_states)
}

fn has_same_kernel(set1: &HashSet<LR1Item>, set2: &HashSet<LR1Item>) -> bool {
    let set1 = set1.iter().map(|x| (&x.left, &x.right, &x.pos)).collect::<HashSet<_>>();
    let set2 = set2.iter().map(|x| (&x.left, &x.right, &x.pos)).collect::<HashSet<_>>();

    set1 == set2
}

#[allow(unused)]
fn lr1_to_lalr1<'a>(item_sets: &'a [HashSet<LR1Item>], next_states: &'a [HashMap<Symbol, usize>]) -> (Vec<HashSet<LR1Item>>, Vec<HashMap<Symbol, usize>>) {
    let mut should_skip = BTreeSet::new();
    let mut idx = 0;

    let mut lalr1_item_sets = Vec::new();
    let mut merged_next_states = Vec::new();
    let mut id_map = BTreeMap::new();

    while idx < item_sets.len() {
        if should_skip.contains(&idx) {
            idx += 1;
            continue;
        }

        let mut item_set = item_sets[idx].clone();
        let mut next_state = next_states[idx].iter().map(|(x, &y)| (x.clone(), y)).collect::<Vec<_>>();
        assert!(id_map.insert(idx, idx).is_none());
        if idx != item_sets.len() - 1 {
            for (j, set2) in item_sets[idx + 1..].iter().enumerate() {
                if should_skip.contains(&j) {
                    continue;
                }
                if has_same_kernel(&item_set, set2) {
                    should_skip.insert(j);
                    id_map.insert(j, idx);
                    item_set.extend(set2.iter().cloned());
                    next_state.extend(next_states[j].iter().map(|(x, &y)| (x.clone(), y)))
                }
            }
        }
        lalr1_item_sets.push(item_set);
        merged_next_states.push(next_state);

        idx += 1;
    }

    let mut lalr1_next_states = Vec::new();

    for (i, ref_next_state) in merged_next_states.into_iter().enumerate() {
        let mut next_state = HashMap::new();
        for (k, v) in ref_next_state {
            let v = id_map[&v];
            if let Some(u) = next_state.insert(k.clone(), v) {
                if u != v {
                    eprintln!("conflict detected while converting I{i} + {k:?}");
                    eprintln!("I{i}:");
                    for item in &lalr1_item_sets[i] {
                        print!("  {} -> ", item.left.symbol_name());
                        for (p, rs) in item.right.iter().enumerate() {
                            if item.pos == p {
                                print!(". ")
                            }
                            print!("{} ", rs.symbol_name());
                        }
                        if item.pos == item.right.len() {
                            print!(". ")
                        }
                        println!("@ NEXT = {}", item.next);
                    }
                    panic!()
                }
                // assert_eq!(u, v, "conflict detected while converting I{i} + {k:?}");
            }
        }
        lalr1_next_states.push(next_state);
    }

    (lalr1_item_sets, lalr1_next_states)
}

fn main() {
    let syntax = syntax_def!(
        Start => StmtList,
        StmtList => Stmt,
        StmtList => Stmt ";",
        StmtList => StmtList ";" Stmt,
        Stmt => IfStmt,
        Stmt => WhileStmt,
        Stmt => AssignStmt,
        Stmt => DefStmt,
        Stmt => Block,
        Stmt => Expr,
        IfStmt => "if" Expr "then" Stmt,
        IfStmt => "if" Expr "then" Stmt "else" Stmt,
        WhileStmt => "while" Expr "do" Stmt,
        AssignStmt => "[id]" ":=" Expr,
        DefStmt => "var" IdList ":" "type",
        Block => "begin" StmtList "end",
        Expr => LogicalExpr,
        LogicalExpr => RelExpr,
        LogicalExpr => LogicalExpr "and/or" RelExpr,
        LogicalExpr => "not" Expr,
        RelExpr => SimpleExpr,
        RelExpr => SimpleExpr "<=>" SimpleExpr,
        SimpleExpr => Term,
        SimpleExpr => SimpleExpr "+/-" Term,
        Term => Factor,
        Term => Term "*" Factor,
        Factor => "(" Expr ")",
        Factor => "[id]",
        Factor => "[num]",
    );
    let first_set = first_set(&syntax);
    let follow_set = follow_set(&syntax, NonTerminal::Start, &first_set);
    let (item_sets, next_states) = lr1_item_sets(&syntax, &first_set);
    // let (item_sets, next_states) = lr1_to_lalr1(&item_sets, &next_states);
    println!("FIRST:\n{:#?}", first_set);
    println!("FOLLOW:\n{:#?}", follow_set);
    println!("LR(1) Item Set:");
    for (idx, item_set) in item_sets.iter().enumerate() {
        println!("[I{idx}]");
        for item in item_set {
            print!("  {} -> ", item.left.symbol_name());
            for (p, rs) in item.right.iter().enumerate() {
                if item.pos == p {
                    print!(". ")
                }
                print!("{} ", rs.symbol_name());
            }
            if item.pos == item.right.len() {
                print!(". ")
            }
            println!("@ NEXT = {}", item.next);
        }
        println!();
        if idx > 10 {
            break;
        }
    }
    println!("NEXT STATES:\n{:?}", next_states);
}
