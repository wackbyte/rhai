Variable Resolver
=================

{{#include ../links.md}}

By default, Rhai looks up access to variables from the enclosing block scope,
working its way outwards until it reaches the top (global) level, then it
searches the [`Scope`] that is passed into the `Engine::eval` call.

There is a built-in facility for advanced users to _hook_ into the variable
resolution service and to override its default behavior.

To do so, provide a closure to the [`Engine`] via the `Engine::on_var` method:

```rust
let mut engine = Engine::new();

// Register a variable resolver.
engine.on_var(|name, index, scope, context| {
    match name {
        "MYSTIC_NUMBER" => Ok(Some((42 as INT).into())),
        // Override a variable - make it not found even if it exists!
        "DO_NOT_USE" => Err(Box::new(
            EvalAltResult::ErrorVariableNotFound(name.to_string(), Position::none())
        )),
        // Silently maps 'chameleon' into 'innocent'.
        "chameleon" => scope.get_value("innocent").map(Some).ok_or_else(|| Box::new(
            EvalAltResult::ErrorVariableNotFound(name.to_string(), Position::none())
        )),
        // Return Ok(None) to continue with the normal variable resolution process.
        _ => Ok(None)
    }
});
```


Returned Values are Constants
----------------------------

Variable values, if any returned, are treated as _constants_ by the script and cannot be assigned to.
This is to avoid needing a mutable reference to the underlying data provider which may not be possible to obtain.

In order to change these variables, it is best to push them into a custom [`Scope`] instead of using
a variable resolver. Then these variables can be assigned to and their updated values read back after
the script is evaluated.


Benefits of Using a Variable Resolver
------------------------------------

1. Avoid having to maintain a custom [`Scope`] with all variables regardless of need (because a script may not use them all).

2. _Short-circuit_ variable access, essentially overriding standard behavior.

3. _Lazy-load_ variables when they are accessed, not up-front. This benefits when the number of variables is very large, when they are timing-dependent, or when they are expensive to load.

4. Rename system variables on a script-by-script basis without having to construct different [`Scope`]'s.


Function Signature
------------------

The function signature passed to `Engine::on_var` takes the following form:

> `Fn(name: &str, index: usize, scope: &Scope, context: &EvalContext)`  
> `-> Result<Option<Dynamic>, Box<EvalAltResult>> + 'static`

where:

* `name: &str` - variable name.

* `index: usize` - an offset from the bottom of the current [`Scope`] that the variable is supposed to reside.
  Offsets start from 1, with 1 meaning the last variable in the current [`Scope`].  Essentially the correct variable is at position `scope.len() - index`.

  If `index` is zero, then there is no pre-calculated offset position and a search through the current [`Scope`] must be performed.

* `scope: &Scope` - reference to the current [`Scope`] containing all variables up to the current evaluation position.

* `context: &EvalContext` - reference to the current evaluation _context_, which exposes the following fields:
  * `context.engine(): &Engine` - reference to the current [`Engine`].
  * `context.namespace(): &Module` - reference to the current _global namespace_ (as a [module]) containing all script-defined functions.
  * `context.call_level(): usize` - the current nesting level of function calls.

### Return Value

The return value is `Result<Option<Dynamic>, Box<EvalAltResult>>` where:

* `Ok(None)` - normal variable resolution process should continue, meaning to continue searching through the [`Scope`].

* `Ok(Some(Dynamic))` - wrapped [`Dynamic`] is taken as the value of the variable, which is treated as a constant.

* `Err(Box<EvalAltResult>)` - error is reflected back to the [`Engine`].
  Normally this is `EvalAltResult::ErrorVariableNotFound` to indicate that the variable does not exist, but it can be any error.