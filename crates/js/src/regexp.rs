//! What older form scripts expect of regular expressions beyond ECMAScript: the legacy `RegExp`
//! statics (`RegExp.$1` … `RegExp.$9`, `lastMatch`, `lastParen`, `leftContext`, `rightContext`,
//! `input` and their `$&` `$+` `` $` `` `$'` `$_` aliases), which Acrobat keeps and which many
//! government forms' Format scripts read after a `test` (an ATO form shows a TFN as
//! `RegExp.$1 + " " + RegExp.$2 + " " + RegExp.$3`), and `AFExactMatch`, which their Keystroke
//! scripts call to check a whole entry against a list of patterns.
//!
//! boa has no legacy statics. Every built-in that matches (`test`, `exec`, `String.prototype`
//! `match`, `replace`, `search`, `split`) goes through `RegExp.prototype.exec` (ECMA-262
//! RegExpExec), so wrapping that one method records each successful match.

use boa_engine::object::builtins::JsRegExp;
use boa_engine::property::PropertyDescriptor;

use super::host::{arg, error, function, s};
use super::{Context, JsObject, JsResult, JsString, JsValue, NativeFunction, js_string};

/// The statics a successful match sets, by name and alias. Before any match they are empty.
const GROUPS: [&str; 9] = ["$1", "$2", "$3", "$4", "$5", "$6", "$7", "$8", "$9"];
const LAST_MATCH: [&str; 2] = ["lastMatch", "$&"];
const LAST_PAREN: [&str; 2] = ["lastParen", "$+"];
const LEFT_CONTEXT: [&str; 2] = ["leftContext", "$`"];
const RIGHT_CONTEXT: [&str; 2] = ["rightContext", "$'"];
const INPUT: [&str; 2] = ["input", "$_"];

/// Most patterns `AFExactMatch` tries: a script's list longer than this is cut there.
const MAX_PATTERNS: u64 = 1024;

/// Install the legacy statics on the realm's `RegExp` and `AFExactMatch` as a global.
pub(crate) fn install(ctx: &mut Context) -> JsResult<()> {
    let regexp = ctx.intrinsics().constructors().regexp().constructor();
    let prototype = ctx.intrinsics().constructors().regexp().prototype();
    for name in GROUPS.iter().chain(&LAST_MATCH).chain(&LAST_PAREN).chain(&LEFT_CONTEXT).chain(&RIGHT_CONTEXT).chain(&INPUT) {
        regexp.set(JsString::from(*name), s(""), false, ctx)?;
    }
    let original = prototype.get(js_string!("exec"), ctx)?.as_object().ok_or_else(|| error("RegExp.prototype.exec is missing"))?;
    let exec = function(ctx, NativeFunction::from_copy_closure_with_captures(exec_recording, original));
    prototype.define_property_or_throw(
        js_string!("exec"),
        PropertyDescriptor::builder().value(exec).writable(true).enumerable(false).configurable(true),
        ctx,
    )?;
    ctx.register_global_callable(js_string!("AFExactMatch"), 2, NativeFunction::from_fn_ptr(af_exact_match))?;
    Ok(())
}

/// `RegExp.prototype.exec`, which also records a successful match in the legacy statics.
fn exec_recording(this: &JsValue, args: &[JsValue], original: &JsObject, ctx: &mut Context) -> JsResult<JsValue> {
    let result = original.call(this, args, ctx)?;
    if let Some(m) = result.as_object() {
        record(&m, ctx)?;
    }
    Ok(result)
}

/// Set the legacy statics from match result `m` (an `exec` result: the match, its groups,
/// `index` and `input`). A group that took no part in the match reads as empty.
fn record(m: &JsObject, ctx: &mut Context) -> JsResult<()> {
    let item = |i: u32, ctx: &mut Context| -> JsResult<JsString> {
        let v = m.get(i, ctx)?;
        if v.is_undefined() { Ok(JsString::default()) } else { v.to_string(ctx) }
    };
    let matched = item(0, ctx)?;
    let length = m.get(js_string!("length"), ctx)?.to_length(ctx)?;
    let mut groups = Vec::with_capacity(GROUPS.len());
    for i in 1..=GROUPS.len() as u32 {
        groups.push(if u64::from(i) < length { item(i, ctx)? } else { JsString::default() });
    }
    // The last group of the pattern, whether or not it took part.
    let last_paren = match u32::try_from(length.saturating_sub(1)) {
        Ok(last) if last > 0 => item(last, ctx)?,
        _ => JsString::default(),
    };
    let input = m.get(js_string!("input"), ctx)?.to_string(ctx)?;
    let index = usize::try_from(m.get(js_string!("index"), ctx)?.to_length(ctx)?).unwrap_or(usize::MAX).min(input.len());
    let end = index.saturating_add(matched.len()).min(input.len());
    let left = input.get(..index).unwrap_or_default();
    let right = input.get(end..).unwrap_or_default();

    let regexp = ctx.intrinsics().constructors().regexp().constructor();
    let set = |names: &[&str], value: &JsString, ctx: &mut Context| -> JsResult<()> {
        for name in names {
            regexp.set(JsString::from(*name), JsValue::from(value.clone()), false, ctx)?;
        }
        Ok(())
    };
    for (name, value) in GROUPS.iter().zip(&groups) {
        set(&[name], value, ctx)?;
    }
    set(&LAST_MATCH, &matched, ctx)?;
    set(&LAST_PAREN, &last_paren, ctx)?;
    set(&LEFT_CONTEXT, &left, ctx)?;
    set(&RIGHT_CONTEXT, &right, ctx)?;
    set(&INPUT, &input, ctx)?;
    Ok(())
}

/// `AFExactMatch(patterns, text)`: the 1-based position of the first pattern (a regular
/// expression, or an array of them) whose match is the whole of `text`, or 0 when none is.
/// Entries that aren't regular expressions are skipped.
fn af_exact_match(_: &JsValue, args: &[JsValue], ctx: &mut Context) -> JsResult<JsValue> {
    let text_value = arg(args, 1);
    let text = if text_value.is_undefined() || text_value.is_null() { JsString::default() } else { text_value.to_string(ctx)? };
    let patterns = arg(args, 0);
    let Some(list) = patterns.as_object() else { return Ok(JsValue::from(0)) };
    let candidates: Vec<JsValue> = if list.is_array() {
        let n = list.get(js_string!("length"), ctx)?.to_length(ctx)?.min(MAX_PATTERNS);
        let mut out = Vec::new();
        for i in 0..n {
            out.push(list.get(i, ctx)?);
        }
        out
    } else {
        vec![patterns.clone()]
    };
    for (i, pattern) in candidates.iter().enumerate() {
        let Some(re) = pattern.as_object().filter(|o| JsRegExp::from_object(o.clone()).is_ok()) else { continue };
        let exec = re.get(js_string!("exec"), ctx)?;
        let Some(exec) = exec.as_callable() else { continue };
        let m = exec.call(pattern, &[JsValue::from(text.clone())], ctx)?;
        if let Some(m) = m.as_object()
            && m.get(0u32, ctx)?.to_string(ctx)? == text
        {
            return Ok(JsValue::from(u32::try_from(i).unwrap_or(u32::MAX - 1).saturating_add(1)));
        }
    }
    Ok(JsValue::from(0))
}
