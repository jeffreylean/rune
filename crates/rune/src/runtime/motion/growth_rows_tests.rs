use super::*;
use crate::{GrowthSpec, col, el};

/// A growth stack of rows, the middle one itself a growth stack: `extra` more stacks before it,
/// and with `open` a row opening above the inner stack and one inside it.
#[derive(Clone, Copy)]
struct S {
    open: bool,
    extra: usize,
}

const GAP: f32 = 8.0;
const INNER_GAP: f32 = 6.0;
const INNER_PAD: f32 = 4.0;

fn view(s: &S, _: &mut Cx<S>) -> El {
    let inner = col().key("inner").w_full().padding(INNER_PAD).gap(INNER_GAP).growth(GrowthSpec::default())
        .child(el().key("i0").w_full().h(30.0))
        .children(s.open.then(|| el().key("i1").w_full().h(50.0)))
        .child(el().key("i2").w_full().h(30.0));
    let extra = (0..s.extra).map(|k| col().key(format!("x{k}")).growth(GrowthSpec::default()).child(el().w(10.0).h(10.0)));
    col().w_full().h_full()
        .children(extra)
        .child(col().key("outer").w_full().gap(GAP).growth(GrowthSpec::default())
            .child(el().key("top").w_full().h(40.0))
            .children(s.open.then(|| el().key("new").w_full().h(60.0)))
            .child(inner)
            .child(el().key("after").w_full().h(20.0)))
}

fn settle(rt: &mut Runtime<S>) {
    for _ in 0..240 { rt.frame(1.0 / 120.0); }
    assert!(!rt.animating);
}

/// The tree `S { open: false, extra: 0 }`, reached directly, or after `extra` more stacks came
/// and went, which leaves the runtime's stack table larger and walked in another order.
fn built(extra: usize) -> Runtime<S> {
    let mut rt = Runtime::new(S { open: false, extra }, view, (300.0, 600.0), Env::new(true, 1.0));
    rt.frame(0.0);
    settle(&mut rt);
    rt.update(|s, _| s.extra = 0);
    settle(&mut rt);
    rt
}

fn rect(rt: &Runtime<S>, key: &str) -> [f32; 4] { rt.presented(key).unwrap().0 }

/// The order the runtime walks its growth stacks in, by key.
fn walk(rt: &Runtime<S>) -> Vec<String> {
    rt.stacks.keys().filter_map(|id| rt.nodes[*rt.node_index.get(id)?].key.clone()).collect()
}

/// Opens a row above the inner stack and one inside it, and returns each frame's drawn
/// rectangles of the inner stack, its rows and the rows beside it.
fn open(rt: &mut Runtime<S>) -> Vec<Vec<[f32; 4]>> {
    rt.update(|s, _| s.open = true);
    (0..120).map(|_| {
        rt.frame(1.0 / 120.0);
        ["new", "inner", "i0", "i1", "i2", "after"].iter().map(|k| rect(rt, k)).collect()
    }).collect()
}

/// The same tree reached by two histories whose stack tables are walked in different orders.
fn two_histories() -> [Runtime<S>; 2] {
    let fresh = built(0);
    let grown = (1..40).map(built).find(|rt| walk(rt) != walk(&fresh)).expect("some history walks the stacks in another order");
    [fresh, grown]
}

#[test]
fn a_stack_inside_a_stack_draws_in_the_slot_the_outer_stack_gives_it() {
    for (h, mut rt) in two_histories().into_iter().enumerate() {
        for (f, r) in open(&mut rt).iter().enumerate() {
            let [new, inner, i0, _, _, after] = r[..] else { unreachable!() };
            let (top, bottom) = (inner[1], inner[1] + inner[3]);
            assert!((top - (new[1] + new[3] + GAP)).abs() < 0.5, "history {h} frame {f}: the inner stack's top {top} is a gap below the row above, {new:?}");
            assert!((bottom + GAP - after[1]).abs() < 0.5, "history {h} frame {f}: the inner stack's bottom {bottom} is a gap above the row below, {after:?}");
            assert!((i0[1] - (top + INNER_PAD)).abs() < 0.5, "history {h} frame {f}: its first row {i0:?} is its padding below its top {top}");
        }
    }
}

#[test]
fn a_stack_inside_a_stack_draws_the_same_whatever_stacks_came_before() {
    let [mut fresh, mut grown] = two_histories();
    let (a, b) = (open(&mut fresh), open(&mut grown));
    for (f, (a, b)) in a.iter().zip(&b).enumerate() {
        assert_eq!(a, b, "frame {f}: new, inner, i0, i1, i2, after");
    }
}
