use egg::*;
use egg::udp_lang::USr;
use crate::udp_analysis::*;
use crate::udp_analysis::UAnalysis;
use crate::udp_rewrites::*;

fn prove_eqs(exprs: &[&str], rls: &[Rewrite<USr, UAnalysis>]) {
    let mut runner = Runner::default();
    for e in exprs {
        runner = runner.with_expr(&e.parse().unwrap());
    }
    runner = runner.run(rls);
    // runner.egraph.dot().to_dot("g.dot").unwrap();
    for (i, _) in exprs.iter().enumerate() {
        assert_eq!(runner.egraph.find(runner.roots[0]), runner.egraph.find(runner.roots[i]));
    }
}

#[test]
fn eq_11() {
    let mut rls = rules();
    rls.push(rewrite!("KEY";"(* ([] (= ?vtk ?vttk)) (* (R ?vtk ?vta) (R ?vttk ?vtta)))"
                            =>
                            "(* (* ([] (= ?vtk ?vttk)) ([] (= ?vta ?vtta))) (R ?vtk ?vta))"));
    // TODO add let-rel rules
    rls.push(rewrite!("let-R"; "(let ?v ?e (R ?k ?a))" => "(R (let ?v ?e ?k) (let ?v ?e ?a))"));
    prove_eqs(&vec![
        "(* (R (var ttk) (var tta)) 
            (sig tk 
                (sig ta 
                    (* (R (var tk) (var ta)) 
                        ([] (= (var tk) (var ttk)))))))",
        // push down (R (var ttk) (var tta))
        "(sig tk 
            (sig ta 
                (* ([] (= (var tk) (var ttk))) 
                    (* (R (var tk) (var ta)) 
                        (R (var ttk) (var tta))))))",
        // apply key constraint
        "(sig tk 
            (sig ta 
                (* (* ([] (= (var tk) (var ttk))) 
                        ([] (= (var ta) (var tta)))) 
                    (R (var tk) (var ta)))))",
        // pull up ([] (= (var tk) (var ttk)))
        "(sig tk 
            (* ([] (= (var tk) (var ttk)))  
                (sig ta 
                    (* ([] (= (var ta) (var tta))) 
                        (R (var ttk) (var tta))))))",
        // eliminate sums
        "(R (var ttk) (var tta))",
    ], &rls);
}

#[test]
fn lemma_5_1() {
    prove_eqs(&vec![
            "(|| (+ (* (var a) (|| (var x))) (var y)))",
            "(|| (+ (* (var a) (var x)) (var y)))",
        ], &rules())
}

#[test]
fn equality_semantics() {
    prove_eqs(&vec!["(sig t (* (var t) ([] (= (var t) (var e)))))","(var e)"], &rules())
}

#[test]
fn spnf() {
    prove_eqs(&vec![
        "(sig t1 (sig t2 (sig t3
            (* ([] (= t2 t)) 
                (* ([] (= (. t1 k) (. t2 k)))
                    (* ([] (>= (. t1 a) 12)) 
                        (* ([] (= (. t3 k) (. t1 k))) 
                            (* ([] (= (. t3 a) (. t1 a))) 
                                (* (R t3) 
                                    (R t2))))))))))",
        "(sig t1 
            (sig t2
                (* ([] (= t2 t)) 
                    (* (* ([] (= (. t1 k) (. t2 k))) ([] (>= (. t1 a) 12))) 
                        (sig t3 
                            (* (* ([] (= (. t3 k) (. t1 k))) (* ([] (= (. t3 a) (. t1 a))) (R t3))) 
                                (R t2)))))))",
        "(sig t1 (sig t2 
            (* ([] (= t2 t)) 
                (* (sig t3 (* ([] (= (. t3 k) (. t1 k))) 
                            (* ([] (= (. t3 a) (. t1 a))) (R t3))))
                    (* (R t2) 
                        (* ([] (= (. t1 k) (. t2 k)))
                            ([] (>= (. t1 a) 12))))))))",
    ], &rules())
}
