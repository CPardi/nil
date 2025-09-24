//! Remove an unused `rec`
//!
//! ```nix
//! rec { a = 1; }
//! ```
//! =>
//! ```nix
//! { a = 1; }
//! ```
use super::{AssistKind, AssistsCtx};
use crate::DiagnosticKind::UnusedRec;
use crate::TextEdit;
use syntax::ast;

pub(super) fn remove_unused_rec(ctx: &mut AssistsCtx<'_>) -> Option<()> {
    let cursor_attrset = ctx.covering_node::<ast::AttrSet>()?;
    let rec_token = cursor_attrset.rec_token()?;
    let rec_range = rec_token.text_range();

    let file = ctx.frange.file_id;
    let check = ctx.db.liveness_check(file);
    let diags = check.as_ref().to_diagnostics(ctx.db, file);

    let no_relevant_diags = diags
        .filter(|d| d.kind == UnusedRec && d.range.intersect(rec_range).is_some())
        .count()
        == 0;

    if no_relevant_diags {
        return None;
    }

    let trivia_range = std::iter::successors(rec_token.next_token(), |tok| tok.next_token())
        .take_while(|tok| tok.kind().is_trivia())
        .last()
        .unwrap_or(rec_token);

    ctx.add(
        "remove_unused_rec",
        "Remove unused rec",
        AssistKind::QuickFix,
        vec![TextEdit {
            delete: rec_range.cover(trivia_range.text_range()),
            insert: Default::default(),
        }],
    );

    Some(())
}

#[cfg(test)]
mod tests {
    use expect_test::expect;

    define_check_assist!(super::remove_unused_rec);

    #[test]
    fn in_use_rec() {
        check_no("let a = 1; in $0rec { a = 3; b = a + 1; }");
    }

    #[test]
    fn unused_rec() {
        // Simple
        check("$0rec { a = 1; }", expect!["{ a = 1; }"]);

        // With trivia
        check("$0rec /* trivia */ { a = 3; }", expect!["{ a = 3; }"]);

        // let-in and rec
        check("let a = 1; in $0rec { a = 3; }", expect!["let a = 1; in { a = 3; }"]);
    }
}
