use crate::{rewrite as rw, *};
use crate::udp_analysis::*;
use crate::udp_lang::*;
use crate::{Applier, EGraph, Id, Pattern, PatternAst, Rewrite, Subst, Symbol, Var};

fn var(s: &str) -> Var {
    s.parse().unwrap()
}

fn both(
    f: impl Fn(&mut EGraph<USr, UAnalysis>, Id, &Subst) -> bool,
    g: impl Fn(&mut EGraph<USr, UAnalysis>, Id, &Subst) -> bool,
) -> impl Fn(&mut EGraph<USr, UAnalysis>, Id, &Subst) -> bool {
    move |egraph, id, subst| f(egraph, id, subst) && g(egraph, id, subst)
}

fn is_not_same_var(v1: Var, v2: Var) -> impl Fn(&mut EGraph<USr, UAnalysis>, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph.find(subst[v1]) != egraph.find(subst[v2])
}

fn free(x: Var, b: Var) -> impl Fn(&mut EGraph<USr, UAnalysis>, Id, &Subst) -> bool {
    move |egraph, _, subst| egraph[subst[b]].data.free.contains(&subst[x])
}

fn not_free(x: Var, b: Var) -> impl Fn(&mut EGraph<USr, UAnalysis>, Id, &Subst) -> bool {
    let f = free(x, b);
    move |egraph, id, subst| !f(egraph, id, subst)
}

// Rename summation variable and push down sum
pub struct RenameSig {
    fresh: Var,
    e: Pattern<USr>,
}

impl Applier<USr, UAnalysis> for RenameSig {
    fn apply_one(&self, egraph: &mut EGraph<USr, UAnalysis>, eclass: Id, subst: &Subst, searcher_ast: Option<&PatternAst<USr>>, rule_name: Symbol) -> Vec<Id> {
        let mut subst = subst.clone();
        let sym = egraph.add(USr::Symbol(format!("_{}", eclass).into()));
        subst.insert(self.fresh, sym);
        self.e.apply_one(egraph, eclass, &subst, searcher_ast, rule_name)
    }
}

pub fn rules() -> Vec<Rewrite<USr, UAnalysis>> {
    // USr axioms
    let mut rls = vec![
        rw!("assoc-add";   "(+ ?a (+ ?b ?c))" => "(+ (+ ?a ?b) ?c)"),
        rw!("assoc-add-r"; "(+ (+ ?a ?b) ?c)" => "(+ ?a (+ ?b ?c))"),
        rw!("assoc-mul";   "(* ?a (* ?b ?c))" => "(* (* ?a ?b) ?c)"),
        rw!("assoc-mul-r"; "(* (* ?a ?b) ?c)" => "(* ?a (* ?b ?c))"),

        rw!("comm-add";  "(+ ?a ?b)" => "(+ ?b ?a)"),
        rw!("comm-mul";  "(* ?a ?b)" => "(* ?b ?a)"),
        
        rw!("zero-add"; "(+ ?a 0)" => "?a"),
        rw!("zero-mul"; "(* ?a 0)" => "0"),
        rw!("one-mul";  "(* ?a 1)" => "?a"),
    
        rw!("add-zero"; "?a" => "(+ ?a 0)"),
        rw!("mul-one";  "?a" => "(* ?a 1)"),

        rw!("distribute"; "(* ?a (+ ?b ?c))" => "(+ (* ?a ?b) (* ?a ?c))"),
        rw!("factor"    ; "(+ (* ?a ?b) (* ?a ?c))" => "(* ?a (+ ?b ?c))"),    
    ];

    // let rules
    rls.extend(vec![
        rw!("let-const"; "(let ?v ?e ?c)" => "?c" if not_free(var("?v"), var("?c"))),
        rw!("let-var-same"; "(let ?v ?e (var ?v))" => "?e"),
        rw!("let-var-diff"; "(let ?v1 ?e (var ?v2))" => "(var ?v2)"
            if is_not_same_var(var("?v1"), var("?v2"))),
        rw!("let-sig-same"; "(let ?v1 ?e (sig ?v1 ?body))" => "(sig ?v1 ?body)"),
        rw!("let-sig-diff-free"; 
            "(let ?v1 ?e (sig ?v2 ?body))" => 
            { RenameSig {
                fresh: var("?fresh"),
                e: "(sig ?fresh (let ?v1 ?e (let ?v2 ?fresh ?body)))".parse().unwrap()
            }}
            if both(is_not_same_var(var("?v1"), var("?v2")), free(var("?v2"), var("?e")))
        ),
        rw!("let-sig-diff-bound"; 
            "(let ?v1 ?e (sig ?v2 ?body))" => "(sig ?v2 (let ?v1 ?e ?body))"
            if both(is_not_same_var(var("?v1"), var("?v2")), not_free(var("?v2"), var("?e")))
        ),
        rw!("let-add";    "(let ?v ?e (+ ?a ?b))" => "(+ (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-add-r";  "(+ (let ?v ?e ?a) (let ?v ?e ?b))" => "(let ?v ?e (+ ?a ?b))"),
        rw!("let-eq";     "(let ?v ?e (= ?a ?b))" => "(= (let ?v ?e ?a) (let ?v ?e ?b))"),
        rw!("let-eq-r";   "(= (let ?v ?e ?a) (let ?v ?e ?b))" => "(let ?v ?e (= ?a ?b))"),
    ]);

    // squash axioms
    rls.extend(vec![
        rw!("1-a";   "(|| 0)" => "0"),
        rw!("1-a-r"; "0" => "(|| 0)"),
        rw!("1-b"; "(|| (+ 1 ?x))" => "1"),
        rw!("2";   "(|| (+ (|| ?x) ?y))" => "(|| (+ ?x ?y))"),
        rw!("2-r"; "(|| (+ ?x ?y))" => "(|| (+ (|| ?x) ?y))"),
        rw!("3";   "(* (|| ?x) (|| ?y))" => "(|| (* ?x ?y))"),
        rw!("3-r"; "(|| (* ?x ?y))" => "(* (|| ?x) (|| ?y))"),
        rw!("4";   "(* (|| ?x) (|| ?x))" => "(|| ?x)"),
        rw!("4-r"; "(|| ?x)" => "(* (|| ?x) (|| ?x))"),
        rw!("5";   "(* ?x (|| ?x))" => "?x"),
        rw!("5-r"; "?x" => "(* ?x (|| ?x))"),
        rw!("6";   "(|| ?x)" => "?x" if ConditionEqual::parse("?x", "(* ?x ?x))")),
        rw!("6-r"; "?x" => "(|| ?x)" if ConditionEqual::parse("?x", "(* ?x ?x))")),
    ]);

    // negation axioms
    rls.extend(vec![
        rw!("n-1";   "(not 0)" => "1"),
        rw!("n-1-r"; "1" => "(not 0)"),
        rw!("n-2";   "(not (* ?x ?y))" => "(|| (+ (not ?x) (not ?y)))"),
        rw!("n-2-r"; "(|| (+ (not ?x) (not ?y)))" => "(not (* ?x ?y))"),
        rw!("n-3";   "(not (+ ?x ?y))" => "(* (not ?x) (not ?y))"),
        rw!("n-3-r"; "(* (not ?x) (not ?y))" => "(not (+ ?x ?y))"),
        rw!("n-4-a"; "(not (|| ?x))" => "(|| (not ?x))"),
        rw!("n-4-b"; "(|| (not ?x))" => "(not ?x)"),
        rw!("n-4-c"; "(not ?x)" => "(not (|| ?x))"),
    ]);

    // summation axioms
    rls.extend(vec![
        rw!("7";   "(sig ?t (+ ?a ?b))" => "(+ (sig ?t ?a) (sig ?t ?b))"),
        rw!("7-r"; "(+ (sig ?t ?a) (sig ?t ?b))" => "(sig ?t (+ ?a ?b))"),
        rw!("8"; "(sig ?s (sig ?t ?a))" => "(sig ?s (sig ?t ?a))"),
        rw!("9-bound";   "(* ?b (sig ?x ?a))" => "(sig ?x (* ?b ?a))"
                if not_free(var("?x"), var("?b"))),
        rw!("9-bound-r"; "(sig ?x (* ?b ?a))" => "(* ?b (sig ?x ?a))"
                if not_free(var("?x"), var("?b"))),
        rw!("9-free";
            "(* ?b (sig ?x ?a))" =>
            { RenameSig {
                fresh: var("?fresh"),
                e: "(sig ?fresh (* ?b (let ?x ?fresh ?a)))".parse().unwrap()
            }}
            if free(var("?x"), var("?b"))),
        rw!("10";   "(|| (sig ?t ?a))" => "(|| (sig ?t (|| ?a)))"),
        rw!("10-r"; "(|| (sig ?t (|| ?a)))" => "(|| (sig ?t ?a))"),
    ]);

    // conditional axioms
    rls.extend(vec![
        rw!("eq-comm"; "(= ?x ?y)" => "(= ?y ?x)"),
        rw!("neq";   "(not (= ?x ?y))" => "(!= ?x ?y)"),
        rw!("neq-r"; "(!= ?x ?y)" => "(not (= ?x ?y))"),

        rw!("11";   "([] ?b)" => "(|| ([] ?b))"),
        rw!("11-r";   "(|| ([] ?b))" => "([] ?b)"),
        rw!("12"; "(+ ([] (= ?a ?b)) ([] (!= ?a ?b)))"=>"1"),
        rw!("13"; "(* ?e ([] (= (var ?x) ?y)))" => "(* (let ?x ?y ?e) ([] (= (var ?x) ?y)))"),
        rw!("14"; "(sig ?t ([] (= (var ?t) ?e)))" => "1" if not_free(var("?t"), var("?e"))),
    ]);
    
    rls
}
